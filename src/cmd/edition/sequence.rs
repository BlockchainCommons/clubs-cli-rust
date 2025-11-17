use anyhow::{Context, Result, anyhow, bail};
use bc_components::XID;
use bc_envelope::prelude::*;
use clap::Args;
use clubs::provenance_mark_provider::ProvenanceMarkProvider;
use known_values::{HAS_RECIPIENT_RAW, PROVENANCE_RAW, SIGNED_RAW};
use provenance_mark::ProvenanceMark;

use crate::io;

#[derive(Clone)]
struct EditionSummary {
    club_xid: XID,
    provenance: ProvenanceMark,
}

impl clubs::provenance_mark_provider::ProvenanceMarkProvider
    for EditionSummary
{
    fn provenance_mark(&self) -> &ProvenanceMark { &self.provenance }
}

/// Validate that a group of editions share the same club and form a contiguous
/// provenance chain.
#[derive(Debug, Args)]
pub struct CommandArgs {
    /// Edition URs to inspect.
    #[arg(long = "edition", value_name = "UR", required = true)]
    pub editions: Vec<String>,
}

pub fn exec(args: CommandArgs) -> Result<()> {
    if args.editions.len() < 2 {
        bail!("at least two editions are required");
    }

    let mut summaries: Vec<EditionSummary> =
        Vec::with_capacity(args.editions.len());
    for (index, spec) in args.editions.iter().enumerate() {
        let envelope = io::parse_envelope(spec).with_context(|| {
            format!("failed to parse edition at position {}", index + 1)
        })?;

        let summary = extract_summary(envelope).with_context(|| {
            format!(
                "input edition at position {} is not a valid club edition",
                index + 1
            )
        })?;

        summaries.push(summary);
    }

    let first = &summaries[0];
    let first_club = first.club_xid;
    if summaries
        .iter()
        .any(|edition| edition.club_xid != first_club)
    {
        bail!("editions reference multiple clubs");
    }

    let first_chain = first.provenance.chain_id().to_vec();
    if summaries
        .iter()
        .any(|edition| edition.provenance.chain_id() != first_chain.as_slice())
    {
        bail!("editions originate from different provenance chains");
    }

    let mut sorted: Vec<&EditionSummary> = summaries.iter().collect();
    sorted.sort_by_key(|edition| edition.provenance.seq());

    let mut breaks = Vec::new();
    for pair in sorted.windows(2) {
        if !pair[0].precedes(pair[1]) {
            breaks.push((pair[0].provenance.seq(), pair[1].provenance.seq()));
        }
    }

    for (prev, next) in &breaks {
        eprintln!(
            "warning: provenance break between seq {} and {}",
            prev, next
        );
    }

    if let Some(first_sorted) = sorted.first()
        && !first_sorted.provenance.is_genesis()
    {
        eprintln!(
            "warning: sequence starts at seq {}",
            first_sorted.provenance.seq()
        );
    }

    Ok(())
}

fn extract_summary(mut envelope: Envelope) -> Result<EditionSummary> {
    loop {
        if envelope.check_type_envelope("Edition").is_ok() {
            break;
        }

        if envelope
            .optional_assertion_with_predicate(known_values::SIGNED)?
            .is_some()
        {
            envelope = envelope.subject();
            continue;
        }

        if envelope.is_wrapped() {
            envelope = envelope.try_unwrap()?;
            continue;
        }

        bail!("edition envelope does not contain an Edition payload");
    }

    let mut provenance: Option<ProvenanceMark> = None;
    let mut club: Option<XID> = None;

    for assertion in envelope.assertions() {
        let predicate = assertion.try_predicate()?;

        if predicate == Envelope::new("club") {
            let obj = assertion.try_object()?;
            if obj.is_obscured() {
                bail!("club assertion is obscured");
            }
            club = Some(obj.extract_subject::<XID>()?);
            continue;
        }

        if let Ok(kv) = predicate.try_known_value() {
            match kv.value() {
                PROVENANCE_RAW => {
                    if provenance.is_some() {
                        bail!("multiple provenance marks");
                    }
                    let obj = assertion.try_object()?;
                    provenance = Some(ProvenanceMark::try_from(obj.clone())?);
                }
                SIGNED_RAW | HAS_RECIPIENT_RAW => {}
                _ => {}
            }
        }
    }

    let provenance =
        provenance.ok_or_else(|| anyhow!("missing provenance mark"))?;
    let club = club.ok_or_else(|| anyhow!("missing club assertion"))?;

    Ok(EditionSummary { club_xid: club, provenance })
}
