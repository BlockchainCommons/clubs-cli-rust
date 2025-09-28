use anyhow::{Context, Result, anyhow, bail};
use bc_components::{
    PrivateKeys, ReferenceProvider, SSKRGroupSpec, SSKRSpec, XIDProvider,
};
use bc_ur::UREncodable;
use bc_xid::XIDDocument;
use clap::Args;
use clubs::{
    edition::Edition, provenance_mark_provider::ProvenanceMarkProvider,
    public_key_permit::PublicKeyPermit,
};

use crate::io::{self, RecipientDescriptor};

/// Arguments for composing and signing a club edition.
#[derive(Debug, Args)]
pub struct CommandArgs {
    /// Publisher's XID document UR (must include signing keys).
    #[arg(long, value_name = "UR", global = true)]
    pub publisher: String,
    /// Content envelope UR for this edition.
    #[arg(long, value_name = "UR")]
    pub content: String,
    /// Provenance mark UR bound to this edition.
    #[arg(long, value_name = "UR")]
    pub provenance: String,
    /// Permit descriptors (XID or public-keys UR).
    #[arg(long = "permit", value_name = "UR")]
    pub permits: Vec<String>,
    /// Optional SSKR specifications (e.g. "2of3").
    #[arg(long = "sskr", value_name = "SPEC")]
    pub sskr: Vec<String>,
    /// Previous edition UR to enforce provenance ordering.
    #[arg(long, value_name = "UR")]
    pub previous: Option<String>,
}

pub fn exec(args: CommandArgs) -> Result<()> {
    let CommandArgs {
        publisher,
        content,
        provenance,
        permits,
        sskr,
        previous,
    } = args;

    let publisher_doc = io::parse_xid_document(&publisher)
        .context("failed to load publisher XID document")?;
    let signing_keys = extract_signing_keys(&publisher_doc)?;
    let club_xid = publisher_doc.xid();

    let content_env = io::parse_envelope(&content)
        .context("failed to load edition content envelope")?;
    if content_env.has_assertions() {
        bail!(
            "content envelope still has assertions; supply a subject-only envelope (wrap the content so assertions are removed) to keep the digest stable"
        );
    }
    let provenance_mark = io::parse_provenance_mark(&provenance)
        .context("failed to parse provenance mark")?;

    if let Some(previous_str) = previous.as_ref() {
        let previous_env = io::parse_envelope(previous_str)
            .context("failed to parse previous edition")?;
        let previous_edition = Edition::try_from(previous_env)
            .context("previous edition input is not a valid club edition")?;
        if !previous_edition.precedes(&provenance_mark) {
            bail!(
                "provided provenance mark does not follow the previous edition's provenance mark"
            );
        }
    }

    let mut recipient_permits: Vec<PublicKeyPermit> = Vec::new();
    for permit_input in permits.iter() {
        let descriptor = io::parse_recipient_descriptor(permit_input)
            .with_context(|| {
                format!("failed to parse permit input '{permit_input}'")
            })?;
        let (permit, _label) = permit_from_descriptor(descriptor);
        recipient_permits.push(permit);
    }

    let sskr_spec = parse_sskr_spec(&sskr)?;

    let edition = Edition::new(club_xid, provenance_mark.clone(), content_env)
        .context("content envelope must not contain assertions")?;
    let (signed_edition, share_groups) = edition
        .seal_with_permits(&recipient_permits, sskr_spec.clone(), &signing_keys)
        .context("failed to compose edition")?;

    let edition_ur = signed_edition.ur_string();
    println!("{}", edition_ur);

    if let Some(groups) = share_groups {
        for group in groups {
            for share in group {
                let ur = share.ur_string();
                println!("{}", ur);
            }
        }
    }

    Ok(())
}

fn extract_signing_keys(doc: &XIDDocument) -> Result<PrivateKeys> {
    if let Some(keys) = doc
        .inception_key()
        .and_then(|key| key.private_keys().cloned())
    {
        return Ok(keys);
    }

    for key in doc.keys() {
        if let Some(private_keys) = key.private_keys() {
            return Ok(private_keys.clone());
        }
    }

    bail!("publisher XID document must include private keys for signing");
}

fn permit_from_descriptor(
    descriptor: RecipientDescriptor,
) -> (PublicKeyPermit, String) {
    if let Some(member_xid) = descriptor.member_xid() {
        let permit =
            PublicKeyPermit::for_member(member_xid, descriptor.public_keys());
        let label = member_xid.to_string();
        (permit, label)
    } else {
        let public_keys = descriptor.public_keys();
        let reference = public_keys.reference();
        let permit = PublicKeyPermit::for_recipient(public_keys);
        let label = reference.to_string();
        (permit, label)
    }
}

fn parse_sskr_spec(values: &[String]) -> Result<Option<SSKRSpec>> {
    if values.is_empty() {
        return Ok(None);
    }

    let mut group_specs: Vec<SSKRGroupSpec> = Vec::new();
    let mut group_threshold: Option<usize> = None;

    for value in values {
        for part in value.split(',') {
            let entry = part.trim();
            if entry.is_empty() {
                continue;
            }

            if let Some((key, value)) = entry.split_once('=') {
                let key = key.trim().to_ascii_lowercase();
                let threshold_value = value.trim();
                if matches!(
                    key.as_str(),
                    "threshold" | "group-threshold" | "group_threshold"
                ) {
                    let parsed = threshold_value
                        .parse::<usize>()
                        .map_err(|err| anyhow!("invalid SSKR group threshold '{threshold_value}': {err}"))?;
                    group_threshold = Some(parsed);
                } else {
                    bail!("unrecognized SSKR option '{key}'");
                }
                continue;
            }

            let spec = parse_group_spec(entry)?;
            group_specs.push(spec);
        }
    }

    if group_specs.is_empty() {
        bail!(
            "at least one SSKR group specification is required when --sskr is provided"
        );
    }

    let threshold = group_threshold.unwrap_or(1);
    let spec = SSKRSpec::new(threshold, group_specs)
        .map_err(|err| anyhow!("invalid SSKR specification: {err}"))?;
    Ok(Some(spec))
}

fn parse_group_spec(input: &str) -> Result<SSKRGroupSpec> {
    let cleaned = input.replace(' ', "").to_ascii_lowercase();
    let (threshold_str, count_str) =
        cleaned.split_once("of").ok_or_else(|| {
            anyhow!("SSKR group spec '{input}' must be in the form MofN")
        })?;
    let member_threshold = threshold_str.parse::<usize>().map_err(|err| {
        anyhow!("invalid SSKR group threshold '{threshold_str}': {err}")
    })?;
    let member_count = count_str.parse::<usize>().map_err(|err| {
        anyhow!("invalid SSKR group count '{count_str}': {err}")
    })?;

    let spec = SSKRGroupSpec::new(member_threshold, member_count)
        .map_err(|err| anyhow!("invalid SSKR group spec '{input}': {err}"))?;
    Ok(spec)
}
