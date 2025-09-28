use anyhow::{Context, Result, anyhow, bail};
use bc_components::{PrivateKeys, SymmetricKey};
use bc_envelope::prelude::Envelope;
use bc_ur::UREncodable;
use clap::Args;
use clubs::edition::Edition;
use dcbor::{CBORTaggedDecodable, prelude::CBOR};

use crate::io;

/// Decrypt edition content using permits, SSKR shards, or raw keys.
#[derive(Debug, Args)]
pub struct CommandArgs {
    /// Edition UR containing the encrypted content.
    #[arg(long, value_name = "UR")]
    pub edition: String,
    /// Permit URs capable of unwrapping the content key.
    #[arg(long = "permit", value_name = "UR")]
    pub permits: Vec<String>,
    /// SSKR share URs for recovering the content key.
    #[arg(long = "sskr", value_name = "UR")]
    pub shards: Vec<String>,
    /// Symmetric key UR for decrypting the content directly.
    #[arg(long, value_name = "UR")]
    pub key: Option<String>,
    /// Publisher descriptor for signature verification.
    #[arg(long, value_name = "UR", alias = "verifier")]
    pub publisher: Option<String>,
    /// Private-key material for decrypting sealed permits (XID document or
    /// private-keys UR).
    #[arg(long = "identity", value_name = "UR", aliases = ["prvkeys", "private-keys"])]
    pub identities: Vec<String>,
    /// Emit decrypted envelope UR to stdout.
    #[arg(long)]
    pub emit_ur: bool,
}

pub fn exec(args: CommandArgs) -> Result<()> {
    let edition_env =
        io::parse_envelope(&args.edition).context("failed to parse edition")?;

    let verifier_descriptor = match args.publisher.as_ref() {
        Some(spec) => Some(
            io::parse_recipient_descriptor(spec)
                .context("failed to parse verifier input")?,
        ),
        None => None,
    };
    let verifier_keys = verifier_descriptor
        .as_ref()
        .map(|desc| desc.public_keys().clone());

    let inner_envelope = if let Some(ref keys) = verifier_keys {
        edition_env
            .verify(keys)
            .context("failed to verify edition signature")?
    } else {
        edition_env.clone().try_unwrap()?
    };

    let edition = Edition::try_from(inner_envelope.clone())
        .context("edition payload is not a valid club edition")?;

    if let Some(descriptor) = verifier_descriptor.as_ref() {
        if let Some(expected_xid) = descriptor.member_xid() {
            if edition.club_xid != expected_xid {
                bail!(
                    "edition references club XID {} but verifier is {}",
                    edition.club_xid,
                    expected_xid
                );
            }
        }
    }

    let sealed_permits = parse_permits(&args.permits)?;
    let share_envelopes = parse_shards(&args.shards)?;

    let private_keys = parse_private_keys(&args.identities)?;

    let mut symmetric_key: Option<SymmetricKey> = None;

    if let Some(key_spec) = args.key.as_ref() {
        let key = io::parse_symmetric_key(key_spec)
            .context("failed to parse symmetric key input")?;
        symmetric_key = Some(key);
    }

    if !sealed_permits.is_empty() {
        if private_keys.is_empty() {
            bail!(
                "private keys are required to decrypt permits; supply --identity"
            );
        }
        let permit_key =
            recover_key_from_permits(&sealed_permits, &private_keys)?;
        if let Some(existing) = symmetric_key.as_ref() {
            if existing != &permit_key {
                bail!(
                    "conflicting symmetric keys recovered from --key and --permit inputs"
                );
            }
        } else {
            symmetric_key = Some(permit_key);
        }
    }

    let sskr_content = if !share_envelopes.is_empty() {
        let refs: Vec<&Envelope> = share_envelopes.iter().collect();
        let joined =
            Envelope::sskr_join(&refs).context("failed to join SSKR shares")?;
        Some(if joined.is_wrapped() {
            joined
                .try_unwrap()
                .context("failed to unwrap joined SSKR content")?
        } else {
            joined
        })
    } else {
        None
    };

    let key_based_content = if edition.content.is_encrypted() {
        if let Some(ref key) = symmetric_key {
            Some(edition.content.decrypt(key).context(
                "failed to decrypt edition content with symmetric key",
            )?)
        } else {
            None
        }
    } else if edition.content.is_wrapped() {
        Some(
            edition
                .content
                .try_unwrap()
                .context("failed to unwrap cleartext content")?,
        )
    } else {
        Some(edition.content.clone())
    };

    let content_envelope = match (sskr_content, key_based_content) {
        (Some(sskr), Some(from_key)) => {
            if !sskr.is_identical_to(&from_key) {
                bail!(
                    "content recovered from SSKR shares does not match the decrypted edition"
                );
            }
            sskr
        }
        (Some(sskr), None) => sskr,
        (None, Some(from_key)) => from_key,
        (None, None) => {
            bail!(
                "unable to recover content; provide SSKR shares or a symmetric key"
            );
        }
    };

    if args.emit_ur {
        println!("{}", content_envelope.ur_string());
    }

    Ok(())
}

fn parse_permits(
    inputs: &[String],
) -> Result<Vec<bc_components::SealedMessage>> {
    let mut permits = Vec::with_capacity(inputs.len());
    for permit in inputs {
        let sealed = io::parse_sealed_message(permit)
            .with_context(|| format!("failed to parse permit '{permit}'"))?;
        permits.push(sealed);
    }
    Ok(permits)
}

fn parse_shards(inputs: &[String]) -> Result<Vec<Envelope>> {
    let mut shares = Vec::with_capacity(inputs.len());
    for shard in inputs {
        let envelope = io::parse_envelope(shard)
            .with_context(|| format!("failed to parse SSKR share '{shard}'"))?;
        shares.push(envelope);
    }
    Ok(shares)
}

fn parse_private_keys(inputs: &[String]) -> Result<Vec<PrivateKeys>> {
    let mut keys = Vec::with_capacity(inputs.len());
    for identity in inputs {
        let parsed = io::parse_private_keys(identity).with_context(|| {
            format!("failed to parse private keys from '{identity}'")
        })?;
        keys.push(parsed);
    }
    Ok(keys)
}

fn recover_key_from_permits(
    permits: &[bc_components::SealedMessage],
    private_keys: &[PrivateKeys],
) -> Result<SymmetricKey> {
    let mut recovered: Option<SymmetricKey> = None;

    for permit in permits {
        for keys in private_keys {
            match permit.decrypt(keys) {
                Ok(data) => {
                    let cbor = match CBOR::try_from_data(&data) {
                        Ok(value) => value,
                        Err(err) => {
                            let preview =
                                hex::encode(&data[..data.len().min(32)]);
                            return Err(anyhow!(
                                "permit decrypted to invalid CBOR data: {err}; preview={preview}"
                            ));
                        }
                    };
                    let symmetric_key = <SymmetricKey as CBORTaggedDecodable>::
                        from_tagged_cbor(cbor)
                        .map_err(|err| {
                            anyhow!(
                                "permit decrypted to unexpected payload: {err}"
                            )
                        })?;
                    if let Some(existing) = recovered.as_ref() {
                        if existing != &symmetric_key {
                            bail!(
                                "different permits yielded conflicting symmetric keys"
                            );
                        }
                    } else {
                        recovered = Some(symmetric_key);
                    }
                    break;
                }
                Err(_) => continue,
            }
        }
    }

    recovered.ok_or_else(|| {
        anyhow!(
            "none of the provided permits could be decrypted with the supplied identities"
        )
    })
}
