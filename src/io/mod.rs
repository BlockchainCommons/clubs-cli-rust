use std::{
    fs,
    io::{self, Read},
    panic,
    path::Path,
};

use anyhow::{Context, Result, bail};
use bc_components::{
    PrivateKeyBase, PrivateKeys, PrivateKeysProvider, PublicKeys, SSKRShare,
    SealedMessage, SymmetricKey, XID, XIDProvider,
};
use bc_envelope::prelude::*;
use bc_xid::{HasPermissions, Privilege, XIDDocument};
use known_values::HOLDER;
use provenance_mark::ProvenanceMark;

/// Descriptor for a permit recipient.
pub struct RecipientDescriptor {
    pub_keys: PublicKeys,
    xid_document: Option<XIDDocument>,
    annotated_xid: Option<XID>,
}

impl RecipientDescriptor {
    /// Returns the public keys associated with the descriptor.
    pub fn public_keys(&self) -> &PublicKeys { &self.pub_keys }

    /// Returns the optional XID document if one was provided.
    #[allow(dead_code)]
    pub fn xid_document(&self) -> Option<&XIDDocument> {
        self.xid_document.as_ref()
    }

    /// Returns the annotated member XID, if present.
    pub fn member_xid(&self) -> Option<XID> {
        if let Some(doc) = self.xid_document.as_ref() {
            Some(doc.xid())
        } else {
            self.annotated_xid
        }
    }
}

/// Read input from a required CLI argument.
pub fn load_from_spec(spec: &str) -> Result<String> {
    if spec == "-" {
        return read_stdin();
    }

    if let Some(path) = spec.strip_prefix('@') {
        let path = path.trim();
        if path.is_empty() {
            bail!("expected a file path after '@'");
        }
        if path == "-" {
            return read_stdin();
        }
        let content = fs::read_to_string(Path::new(path))
            .with_context(|| format!("failed to read input file '{path}'"))?;
        let trimmed = content.trim();
        if trimmed.is_empty() {
            bail!("input file '{path}' is empty");
        }
        return Ok(trimmed.to_owned());
    }

    let trimmed = spec.trim();
    if trimmed.is_empty() {
        bail!("empty argument");
    }
    Ok(trimmed.to_owned())
}

fn read_stdin() -> Result<String> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        bail!("no data provided on stdin");
    }
    Ok(trimmed.to_owned())
}

fn tighten_ur(input: &str) -> String { input.split_whitespace().collect() }

/// Load an Envelope, expecting a UR encoding.
pub fn parse_envelope(spec: &str) -> Result<Envelope> {
    let raw = load_from_spec(spec)?;
    decode_envelope(&raw)
}

fn decode_envelope(raw: &str) -> Result<Envelope> {
    let primary = raw.trim();
    if primary.is_empty() {
        bail!("empty envelope input");
    }

    if let Ok(env) = Envelope::from_ur_string(primary) {
        return Ok(env);
    }

    let compact = tighten_ur(primary);
    if compact != primary
        && let Ok(env) = Envelope::from_ur_string(&compact)
    {
        return Ok(env);
    }

    let ur = UR::from_ur_string(compact)
        .with_context(|| "failed to parse envelope UR")?;
    if ur.ur_type_str() != "envelope" {
        bail!(
            "expected UR type 'envelope' but found '{}'",
            ur.ur_type_str()
        );
    }
    Envelope::from_tagged_cbor(ur.cbor())
        .with_context(|| "failed to decode Envelope CBOR")
}

/// Parse a provenance mark from input.
pub fn parse_provenance_mark(spec: &str) -> Result<ProvenanceMark> {
    let raw = load_from_spec(spec)?;
    decode_provenance_mark(&raw)
}

fn decode_provenance_mark(raw: &str) -> Result<ProvenanceMark> {
    let compact = tighten_ur(raw.trim());
    if compact.is_empty() {
        bail!("empty provenance mark input");
    }
    ProvenanceMark::from_ur_string(compact)
        .with_context(|| "failed to parse provenance mark UR")
}

/// Parse an XID document from input.
pub fn parse_xid_document(spec: &str) -> Result<XIDDocument> {
    let raw = load_from_spec(spec)?;
    decode_xid_document(&raw)
}

fn decode_xid_document(raw: &str) -> Result<XIDDocument> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("empty XID document input");
    }

    if let Ok(doc) = XIDDocument::from_ur_string(trimmed) {
        return Ok(doc);
    }

    let compact = tighten_ur(trimmed);
    if compact != trimmed
        && let Ok(doc) = XIDDocument::from_ur_string(&compact)
    {
        return Ok(doc);
    }

    let ur = UR::from_ur_string(compact)
        .with_context(|| "failed to parse XID document UR")?;

    match ur.ur_type_str() {
        "xid" => XIDDocument::from_ur(&ur)
            .with_context(|| "failed to decode XID document from UR"),
        "envelope" => {
            let env = Envelope::from_tagged_cbor(ur.cbor())
                .with_context(|| "failed to decode XID document envelope")?;
            XIDDocument::try_from(env)
                .with_context(|| "failed to convert envelope to XID document")
        }
        other => bail!("unsupported UR type '{other}' for XID document"),
    }
}

fn decode_public_keys(raw: &str) -> Result<PublicKeys> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("empty public keys input");
    }

    if let Ok(keys) = PublicKeys::from_ur_string(trimmed) {
        return Ok(keys);
    }

    let compact = tighten_ur(trimmed);
    if compact != trimmed
        && let Ok(keys) = PublicKeys::from_ur_string(&compact)
    {
        return Ok(keys);
    }

    let ur = UR::from_ur_string(compact)
        .with_context(|| "failed to parse public keys UR")?;
    match ur.ur_type_str() {
        "crypto-pubkeys" => PublicKeys::from_ur(&ur)
            .with_context(|| "failed to decode public keys from UR"),
        other => bail!("unsupported UR type '{other}' for public keys"),
    }
}

/// Parse a recipient descriptor (XID document or public keys).
pub fn parse_recipient_descriptor(spec: &str) -> Result<RecipientDescriptor> {
    let raw = load_from_spec(spec)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("empty recipient descriptor");
    }

    if let Ok(doc) = decode_xid_document(trimmed) {
        let pub_keys = select_public_keys(&doc)?;
        return Ok(RecipientDescriptor {
            pub_keys,
            xid_document: Some(doc),
            annotated_xid: None,
        });
    }

    if let Some((pub_keys, member_xid)) = decode_public_key_permit(trimmed)? {
        return Ok(RecipientDescriptor {
            pub_keys,
            xid_document: None,
            annotated_xid: member_xid,
        });
    }

    let keys = decode_public_keys(trimmed)?;
    Ok(RecipientDescriptor {
        pub_keys: keys,
        xid_document: None,
        annotated_xid: None,
    })
}

fn select_public_keys(doc: &XIDDocument) -> Result<PublicKeys> {
    use bc_xid::Key;

    let keys: Vec<&Key> = doc.keys().iter().collect();
    if let Some(key) = keys.iter().find(|key| {
        key.permissions()
            .allow()
            .iter()
            .any(|privilege| privilege == &Privilege::All)
    }) {
        return Ok(key.public_keys().clone());
    }

    if let Some(key) = keys.first() {
        return Ok(key.public_keys().clone());
    }

    bail!("XID document does not contain any public keys");
}

fn decode_public_key_permit(
    raw: &str,
) -> Result<Option<(PublicKeys, Option<XID>)>> {
    let Ok(envelope) = decode_envelope(raw) else {
        return Ok(None);
    };

    if !envelope.has_type("PublicKeyPermit") {
        return Ok(None);
    }

    let subject = envelope.subject();
    if subject.is_obscured() {
        bail!("public-key permit subject is obscured");
    }
    let public_keys = subject
        .extract_subject::<PublicKeys>()
        .with_context(|| "public-key permit subject must be public keys")?;

    let holder_assertion =
        envelope.optional_assertion_with_predicate(HOLDER)?;
    let holder = match holder_assertion {
        Some(assertion) => Some(assertion.extract_object::<XID>()?),
        None => None,
    };

    let allowed = if holder.is_some() { 1 } else { 0 };
    if envelope.assertions().len() > allowed {
        bail!("public-key permit contains unsupported assertions");
    }

    Ok(Some((public_keys, holder)))
}

/// Parse private keys from either a UR or an XID document containing them.
pub fn parse_private_keys(spec: &str) -> Result<PrivateKeys> {
    let raw = load_from_spec(spec)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("empty private keys input");
    }

    if let Ok(keys) = decode_private_keys(trimmed) {
        return Ok(keys);
    }

    if let Ok(base) = decode_private_key_base(trimmed) {
        return Ok(base.private_keys());
    }

    let doc = decode_xid_document(trimmed)?;
    extract_private_keys(&doc)
        .with_context(|| "XID document does not contain private keys")
}

fn decode_private_keys(raw: &str) -> Result<PrivateKeys> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("empty private keys input");
    }

    if let Ok(keys) = PrivateKeys::from_ur_string(trimmed) {
        return Ok(keys);
    }

    let compact = tighten_ur(trimmed);
    if compact != trimmed
        && let Ok(keys) = PrivateKeys::from_ur_string(&compact)
    {
        return Ok(keys);
    }

    let ur = UR::from_ur_string(compact)
        .with_context(|| "failed to parse private keys UR")?;
    match ur.ur_type_str() {
        "crypto-prvkeys" => PrivateKeys::from_ur(&ur)
            .with_context(|| "failed to decode private keys from UR"),
        other => bail!("unsupported UR type '{other}' for private keys"),
    }
}

fn decode_private_key_base(raw: &str) -> Result<PrivateKeyBase> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("empty private key base input");
    }

    if let Ok(base) = PrivateKeyBase::from_ur_string(trimmed) {
        return Ok(base);
    }

    let compact = tighten_ur(trimmed);
    if compact != trimmed
        && let Ok(base) = PrivateKeyBase::from_ur_string(&compact)
    {
        return Ok(base);
    }

    let ur = UR::from_ur_string(compact)
        .with_context(|| "failed to parse private key base UR")?;
    match ur.ur_type_str() {
        "crypto-prvkey-base" => PrivateKeyBase::from_ur(&ur)
            .with_context(|| "failed to decode private key base from UR"),
        other => bail!("unsupported UR type '{other}' for private key base"),
    }
}

fn extract_private_keys(doc: &XIDDocument) -> Result<PrivateKeys> {
    if let Some(key) =
        doc.inception_key().and_then(|k| k.private_keys().cloned())
    {
        return Ok(key);
    }

    for key in doc.keys() {
        if let Some(private_keys) = key.private_keys() {
            return Ok(private_keys.clone());
        }
    }

    bail!("no private keys available in XID document")
}

/// Parse a standalone XID from UR or canonical string forms.
pub fn parse_xid_value(spec: &str) -> Result<XID> {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        bail!("empty XID value");
    }

    let inner = trimmed
        .strip_prefix("XID(")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(trimmed)
        .trim();

    if let Ok(xid) = XID::from_ur_string(inner) {
        return Ok(xid);
    }

    match panic::catch_unwind(|| XID::from_hex(inner)) {
        Ok(xid) => Ok(xid),
        Err(_) => bail!("failed to parse XID value"),
    }
}

/// Parse a sealed message permit.
pub fn parse_sealed_message(spec: &str) -> Result<SealedMessage> {
    let raw = load_from_spec(spec)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("empty sealed message input");
    }

    if let Ok(sealed) = SealedMessage::from_ur_string(trimmed) {
        return Ok(sealed);
    }

    let compact = tighten_ur(trimmed);
    if compact != trimmed
        && let Ok(sealed) = SealedMessage::from_ur_string(&compact)
    {
        return Ok(sealed);
    }

    let ur = UR::from_ur_string(compact)
        .with_context(|| "failed to parse sealed message UR")?;
    match ur.ur_type_str() {
        "crypto-sealed" => SealedMessage::from_ur(&ur)
            .with_context(|| "failed to decode sealed message from UR"),
        other => bail!("unsupported UR type '{other}' for sealed message"),
    }
}

/// Parse an SSKR share.
#[allow(dead_code)]
pub fn parse_sskr_share(spec: &str) -> Result<SSKRShare> {
    let raw = load_from_spec(spec)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("empty SSKR share input");
    }

    if let Ok(share) = SSKRShare::from_ur_string(trimmed) {
        return Ok(share);
    }

    let compact = tighten_ur(trimmed);
    if compact != trimmed
        && let Ok(share) = SSKRShare::from_ur_string(&compact)
    {
        return Ok(share);
    }

    let ur = UR::from_ur_string(compact)
        .with_context(|| "failed to parse SSKR share UR")?;
    match ur.ur_type_str() {
        "sskr" => SSKRShare::from_ur(&ur)
            .with_context(|| "failed to decode SSKR share from UR"),
        other => bail!("unsupported UR type '{other}' for SSKR share"),
    }
}

/// Parse a symmetric key UR.
pub fn parse_symmetric_key(spec: &str) -> Result<SymmetricKey> {
    let raw = load_from_spec(spec)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("empty symmetric key input");
    }

    if let Ok(key) = SymmetricKey::from_ur_string(trimmed) {
        return Ok(key);
    }

    let compact = tighten_ur(trimmed);
    if compact != trimmed
        && let Ok(key) = SymmetricKey::from_ur_string(&compact)
    {
        return Ok(key);
    }

    let ur = UR::from_ur_string(compact)
        .with_context(|| "failed to parse symmetric key UR")?;
    match ur.ur_type_str() {
        "crypto-key" => SymmetricKey::from_ur(&ur)
            .with_context(|| "failed to decode symmetric key from UR"),
        other => bail!("unsupported UR type '{other}' for symmetric key"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tighten_removes_whitespace() {
        assert_eq!(tighten_ur(" ur:example / data \n"), "ur:example/data");
    }
}
