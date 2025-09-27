use anyhow::{Context, Result, bail};
use bc_components::{PublicKeys, XID};
use bc_envelope::prelude::Envelope;
use bc_ur::UREncodable;
use clap::Args;
use known_values::HOLDER;

use crate::io;

/// Derive a public-key permit from recipient materials.
#[derive(Debug, Args)]
pub struct CommandArgs {
    /// Recipient descriptor (XID document or public-keys UR).
    #[arg(long, value_name = "UR")]
    pub recipient: Vec<String>,
    /// Optional label to annotate the permit holder.
    #[arg(long, value_name = "XID")]
    pub label: Option<String>,
}

pub fn exec(args: CommandArgs) -> Result<()> {
    if args.recipient.is_empty() {
        bail!("at least one --recipient value is required");
    }

    let override_xid = match args.label.as_ref() {
        Some(label) => Some(
            io::parse_xid_value(label)
                .context("failed to parse --label as an XID value")?,
        ),
        None => None,
    };

    for recipient in &args.recipient {
        let descriptor = io::parse_recipient_descriptor(recipient)
            .with_context(|| {
                format!("failed to parse recipient '{recipient}'")
            })?;

        let member_xid = override_xid.or(descriptor.member_xid());
        let public_keys = descriptor.public_keys().clone();
        let envelope = permit_envelope(&public_keys, member_xid);
        println!("{}", envelope.ur_string());
    }

    Ok(())
}

fn permit_envelope(
    public_keys: &PublicKeys,
    member_xid: Option<XID>,
) -> Envelope {
    let mut envelope =
        Envelope::new(public_keys.clone()).add_type("PublicKeyPermit");
    if let Some(xid) = member_xid {
        envelope = envelope.add_assertion(HOLDER, xid);
    }
    envelope
}
