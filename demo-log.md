## Set zsh options

zsh is the default shell on macOS and many Linux systems. This ensures consistent behavior across shells.

```
setopt nobanghist
```

## Checking prerequisites

Verify that all required commands are available, and that their version numbers are compatible.

```
for cmd in cargo envelope provenance clubs; do
  $cmd --version
done

│ cargo 1.87.0 (99624be96 2025-05-06)
│ bc-envelope-cli 0.23.1
│ provenance-mark-cli 0.6.0
│ clubs-cli 0.1.0
```

## Preparing demo workspace

Set up a clean directory for the demo artifacts. All we currently store here is the directory used to track the publisher's provenance mark generator.

```
rm -rf demo && mkdir -p demo
```

## Deriving publisher cryptographic material

Create the publisher's keypairs and XID document.

```
PUBLISHER_PRVKEYS=$(envelope generate prvkeys)
echo $PUBLISHER_PRVKEYS
PUBLISHER_XID=$(envelope xid new "$PUBLISHER_PRVKEYS")
echo $PUBLISHER_XID
envelope format "$PUBLISHER_XID"

│ ur:crypto-prvkey-base/hdcxpkvdsfbztnjecacenlztiylnbdgdsrbkiacyisfzcmlydsptntstfrrhmtjppmtnhdispdvs
│ ur:xid/tpsplftpsotanshdhdcxkelkotemrlolsfsbiadmhyrtfgchkshtoeptqdrnidmurnmelkrsswgysosrlkmnoyaylstpsotansgylftanshfhdcxjkcandzeweeeeektiokiendmgecnghdtingotelgcnneenvdvojzceoscnwssstytansgrhdcxdkuyztwnlubgckvdtkkiswihfrfhzooxlalelyisdivsjnoyjttljzlgtysffdgooycsfncsfglfoycsfptpsotansgtlftansgohdcxqddsdplauebyeeptaskihfinmhjpnnlssebaeylbdlltsnksdkpsasteyabgsoontansgehdcxaeimiedmrdcajeytrsbelkztinnymulotbbbuegetalufnosuylusavoykbeolftoybstpsotansgmhdcxlrfgatlswnfsuozslfpfcllflewdbngylfcfjpiyurvdteynylsgrswflgsefnspkijogtdi
│ XID(7c8ca337) [
│     'key': PublicKeys(48655278) [
│         {
│             'privateKey': PrivateKeys(5c972906)
│         } [
│             'salt': Salt
│         ]
│         'allow': 'All'
│     ]
│ ]
```

## Creating XID document for ALICE

Provision Alice with keys so we can address permits to real members.

```
ALICE_PRVKEYS=$(envelope generate prvkeys)
echo "ALICE_PRVKEYS=$ALICE_PRVKEYS"
ALICE_PUBKEYS=$(envelope generate pubkeys "$ALICE_PRVKEYS")
echo "ALICE_PUBKEYS=$ALICE_PUBKEYS"
ALICE_XID=$(envelope xid new "$ALICE_PRVKEYS")
echo "ALICE_XID=$ALICE_XID"
envelope format "$ALICE_XID"

│ ALICE_PRVKEYS=ur:crypto-prvkey-base/hdcxfepradjeoybbaymkhtdttstlwzdkvabkjlwsrsjotoiytnrdrklbjtbzkedwlbbabazebakb
│ ALICE_PUBKEYS=ur:crypto-pubkeys/lftanshfhdcxnstsfpwpfttpkbimcnhfnbimgowslyzoamgymshedsfssewfkkpdjkpamudatszotansgrhdcxgupssrotmnpagatdaamugudauobgmygensueiaetjoiyhsjnckdyhfhkwyfhdehyhfwtcpcy
│ ALICE_XID=ur:xid/tpsplftpsotanshdhdcxpahtsbpmrlryvalttbdadnvsgdmteesgbatdykyndidwzswnbsjkfzlohsrpylstoyaylstpsotansgylftanshfhdcxnstsfpwpfttpkbimcnhfnbimgowslyzoamgymshedsfssewfkkpdjkpamudatszotansgrhdcxgupssrotmnpagatdaamugudauobgmygensueiaetjoiyhsjnckdyhfhkwyfhdehyoycsfncsfglfoycsfptpsotansgtlftansgohdcxgalukbdpdymhwnsbtkwylnyaregydlgwclclnycnvtcycavlyastmdoxvsfxzsnttansgehdcxmnyngrhfeopeetclsksoamctgdcyvtiopllpkpgrztuokkascflytenycmvtrlmyoybstpsotansgmhdcxrsfmvyatjkpswnnseniycpaxioingtenonfgahiddmsgyaehrpdetisbsbdmmuvomufmreet
│ XID(b15acbad) [
│     'key': PublicKeys(4cfde8ac) [
│         {
│             'privateKey': PrivateKeys(5886d0f9)
│         } [
│             'salt': Salt
│         ]
│         'allow': 'All'
│     ]
│ ]
```

## Creating XID document for BOB

Provision Bob with keys so we can address permits to real members.

```
BOB_PRVKEYS=$(envelope generate prvkeys)
echo "BOB_PRVKEYS=$BOB_PRVKEYS"
BOB_PUBKEYS=$(envelope generate pubkeys "$BOB_PRVKEYS")
echo "BOB_PUBKEYS=$BOB_PUBKEYS"
BOB_XID=$(envelope xid new "$BOB_PRVKEYS")
echo "BOB_XID=$BOB_XID"
envelope format "$BOB_XID"

│ BOB_PRVKEYS=ur:crypto-prvkey-base/hdcxeocprtktrsjsperleotklrrhtbbgutwnchrppfasmosgnsmdbemdlaayzctnpdflgowzeork
│ BOB_PUBKEYS=ur:crypto-pubkeys/lftanshfhdcxfygrecgdgedlvadwwdhtnyytgscfsfrlceieaydrfyrppejzmelstyglmefgswmstansgrhdcxwydwkthgonnelthhrysgtysofgpylafndwzoladetsnyuodynldamofzbgbkbehlghsfkita
│ BOB_XID=ur:xid/tpsplftpsotanshdhdcxhniepymeqzknwkhfqzeojtecrklsdeeoeogldidszczmchdruewschbkhgwndsfyoyaylstpsotansgylftanshfhdcxfygrecgdgedlvadwwdhtnyytgscfsfrlceieaydrfyrppejzmelstyglmefgswmstansgrhdcxwydwkthgonnelthhrysgtysofgpylafndwzoladetsnyuodynldamofzbgbkbehloycsfncsfglfoycsfptpsotansgtlftansgohdcxfpselgwkdyrllsbsnthnptkohfbeoediswmsfsjodsldecsnpmmhkpfdjpwlneiytansgehdcxrslynttadpbtttaaonwlkgeydsfyetqdlbvllodejyyktetsbzcysedmsadkpmkeoybstpsotansgmhdcxmdylioksutlgtlhdrnaaotcmqdgmjsflidttqzvsbaieaygthpoeurmnvtmupfptynecvdee
│ XID(6064ab91) [
│     'key': PublicKeys(6a83cdc8) [
│         {
│             'privateKey': PrivateKeys(696868ce)
│         } [
│             'salt': Salt
│         ]
│         'allow': 'All'
│     ]
│ ]
```

## Assembling edition content envelope

Wrap the plaintext so its digest remains stable once we start sealing.

```
CONTENT_SUBJECT=$(envelope subject type string 'Welcome to the Gordian Club!')
echo "${CONTENT_SUBJECT}"
envelope format "${CONTENT_SUBJECT}"
echo ""
CONTENT_CLEAR=$(echo "${CONTENT_SUBJECT}" | envelope assertion add pred-obj string "title" string 'Genesis Edition')
echo "${CONTENT_CLEAR}"
envelope format "${CONTENT_CLEAR}"
echo ""
CONTENT_WRAPPED=$(envelope subject type wrapped "${CONTENT_CLEAR}")
echo "${CONTENT_WRAPPED}"
envelope format "${CONTENT_WRAPPED}"

│ ur:envelope/tpsokscehgihjziajljnihcxjyjlcxjyisihcxfljljpieinhsjtcxfxjzkpidclisloctmd
│ "Welcome to the Gordian Club!"
│ 
│ ur:envelope/lftpsokscehgihjziajljnihcxjyjlcxjyisihcxfljljpieinhsjtcxfxjzkpidcloytpsoihjyinjyjzihtpsojlflihjtihjkinjkcxfeieinjyinjljthyurfxat
│ "Welcome to the Gordian Club!" [
│     "title": "Genesis Edition"
│ ]
│ 
│ ur:envelope/tpsplftpsokscehgihjziajljnihcxjyjlcxjyisihcxfljljpieinhsjtcxfxjzkpidcloytpsoihjyinjyjzihtpsojlflihjtihjkinjkcxfeieinjyinjljtialawspd
│ {
│     "Welcome to the Gordian Club!" [
│         "title": "Genesis Edition"
│     ]
│ }
```

## Capturing content digest

Store the content digest that must match the one stored in the Edition's provenance mark's info field.

```
CONTENT_DIGEST=$(envelope digest "${CONTENT_WRAPPED}")
echo "${CONTENT_DIGEST}"

│ ur:digest/hdcxldhndsftresogmkbvebggorslswtiainztdrinkekbltwkpfgwdwjljsfnolgdmezcdidnwt
```

## Starting provenance mark chain

Initialize the publisher's mark generator and bind the genesis mark to the content digest using the info field.

```
GENESIS_MARK=$(provenance new demo/provenance-chain --comment "Genesis edition" --format ur --quiet --info "$CONTENT_DIGEST")
echo "$GENESIS_MARK"
provenance print demo/provenance-chain --start 0 --end 0 --format markdown

│ ur:provenance/lfaohdherpgorymspyyadpwsynpsonimfnlgckskhdtbqdjthnrffptluoecltuthlasnlgukepttnfwdsbypecfcahthhfspawemwtldyghcyaespbbaedyclnychmnrswebscxwdehuthfztettaolmhlndnskdaglfnenladkguhevoztbsdihdolcxinfmjkbgcnsfmkih
│ ---
│ 
│ 2025-10-01T09:17:39Z
│ 
│ #### ur:provenance/lfaohdherpgorymspyyadpwsynpsonimfnlgckskhdtbqdjthnrffptluoecltuthlasnlgukepttnfwdsbypecfcahthhfspawemwtldyghcyaespbbaedyclnychmnrswebscxwdehuthfztettaolmhlndnskdaglfnenladkguhevoztbsdihdolcxinfmjkbgcnsfmkih
│ 
│ #### `🅟 PAID JUDO BETA USER`
│ 
│ 🅟 📌 💨 🤨 🐼
│ 
│ Genesis edition
│ 
```

## Composing genesis edition

Seal the content, attach permits, and sign the first edition with the club keys.

```
EDITION_RAW=$(clubs init \
  --publisher "$PUBLISHER_XID" \
  --content "$CONTENT_WRAPPED" \
  --provenance "$GENESIS_MARK" \
  --permit "$ALICE_XID" \
  --permit "$BOB_PUBKEYS" \
  --sskr 2of3)
print -r -- "$EDITION_RAW"

│ ur:envelope/lftpsplntansfwlrhdfwwzurnnrntdtkrhjnsgwphlvlbzldmypdprbgfzfemoqzsaaytdrdwmwdkorsctsbtkdsfhfhyagllfndbzvdnsrpwywefehpdivecmdicekinbimcsnepkhejpfeksfrvefzgsfrutfhvdrfhlkekbahbkuopmgdlefefmsbgmkohhhghkylkbfxdnzcaomthddatansfphdcxenhtcybkcwtynnkouofpfgrkzsrpwdgddngeetzennzsmwhdcmwfwlpalkylsfpalfoyahtpsotansgulftansfwlrhddaidcytkrkpscettlpvahljkaxdssgdifsgdfybknecahgkprdienbtayleeylldpttsnbtdwlktgsltkkhkaelkhtyktocpfmetcygdhsmkmsvlweatrsjymoglbthlltpagadihdcxrnadfewsspvycheeluldadrhlovyatvwrsghwkkeaspazshtjzclgobsbwbwatpttansgrhdcxehpryavolpeopdtdhkuyjzwpwfhkpsnncavdndfebavdkkwdsgdrndtiwnlemnieoybatpsotanshdhdcxpahtsbpmrlryvalttbdadnvsgdmteesgbatdykyndidwzswnbsjkfzlohsrpylstoyadtpsoiofeieinjyinjljtoycsfztpsotngdgmgwhflfaohdherpgorymspyyadpwsynpsonimfnlgckskhdtbqdjthnrffptluoecltuthlasnlgukepttnfwdsbypecfcahthhfspawemwtldyghcyaespbbaedyclnychmnrswebscxwdehuthfztettaolmhlndnskdaglfnenladkguhevoztbsdihdolcxinfmjkbgoyahtpsotansgulftansfwlrhddasoemflqzwszszestvspacmckhkpsmncnhnehssengahddpytbdlrwmfpwpbzihwegrldpkldspgsjnadwnpfemhpqzosdmdwctlsgdvykklrcptbpypyhyttlpfllfbywttdvohdcxrnadfewsspvycheeluldadrhlovyatvwrsghwkkeaspazshtjzclgobsbwbwatpttansgrhdcxlrspiyrnzcnsseisttvsnsvegetogrryfmampmimsponwknnamjtdezowerpwsdwoytpsoieiajzkpidtpsotanshdhdcxkelkotemrlolsfsbiadmhyrtfgchkshtoeptqdrnidmurnmelkrsswgysosrlkmnoyaxtpsotansghhdfzclvyspjzdicmimsgnefetygarpweneaareuefskprortyaknfpdtdtwktnckwywtmsjzcsfwsfgwrpotfnosaxololcwoxgufxseehotgtntheglrpmswmmtnlskherkahtddkln
│ ur:envelope/lftansfwlrhdfwwzurnnrntdtkrhjnsgwphlvlbzldmypdprbgfzfemoqzsaaytdrdwmwdkorsctsbtkdsfhfhyagllfndbzvdnsrpwywefehpdivecmdicekinbimcsnepkhejpfeksfrvefzgsfrutfhvdrfhlkekbahbkuopmgdlefefmsbgmkohhhghkylkbfxdnzcaomthddatansfphdcxenhtcybkcwtynnkouofpfgrkzsrpwdgddngeetzennzsmwhdcmwfwlpalkylsfpaoyamtpsotantkphddawldyaeadaectdakpkbglcswzhtwebztasevylkjsfwadsnolwnttyanbahdknllgpewfwdrnrofggogyao
│ ur:envelope/lftansfwlrhdfwwzurnnrntdtkrhjnsgwphlvlbzldmypdprbgfzfemoqzsaaytdrdwmwdkorsctsbtkdsfhfhyagllfndbzvdnsrpwywefehpdivecmdicekinbimcsnepkhejpfeksfrvefzgsfrutfhvdrfhlkekbahbkuopmgdlefefmsbgmkohhhghkylkbfxdnzcaomthddatansfphdcxenhtcybkcwtynnkouofpfgrkzsrpwdgddngeetzennzsmwhdcmwfwlpalkylsfpaoyamtpsotantkphddawldyaeadadmtdskeqdpfbgjktddytdnymtpldyfrwmgrswiakopsamrlpezmpsolgteoiyrftagtfxnnjt
│ ur:envelope/lftansfwlrhdfwwzurnnrntdtkrhjnsgwphlvlbzldmypdprbgfzfemoqzsaaytdrdwmwdkorsctsbtkdsfhfhyagllfndbzvdnsrpwywefehpdivecmdicekinbimcsnepkhejpfeksfrvefzgsfrutfhvdrfhlkekbahbkuopmgdlefefmsbgmkohhhghkylkbfxdnzcaomthddatansfphdcxenhtcybkcwtynnkouofpfgrkzsrpwdgddngeetzennzsmwhdcmwfwlpalkylsfpaoyamtpsotantkphddawldyaeadaocmcniozmptbnwmgygslahejllbwsvwbdmduyemvednctmngeldwfuyjoiswlrdknrodtdlbe
```

## Capturing edition artifacts

Inspect the resulting edition and enumerate the emitted SSKR shares.

```
typeset -ga EDITION_URS=("${(@f)${EDITION_RAW%$'\n'}}")
EDITION_UR=${EDITION_URS[1]}
typeset -ga SSKR_URS=("${EDITION_URS[@]:1}")
for ur in "${EDITION_URS[@]}"; do print -r -- "$ur"; envelope format "$ur"; echo ""; done

│ ur:envelope/lftpsplntansfwlrhdfwwzurnnrntdtkrhjnsgwphlvlbzldmypdprbgfzfemoqzsaaytdrdwmwdkorsctsbtkdsfhfhyagllfndbzvdnsrpwywefehpdivecmdicekinbimcsnepkhejpfeksfrvefzgsfrutfhvdrfhlkekbahbkuopmgdlefefmsbgmkohhhghkylkbfxdnzcaomthddatansfphdcxenhtcybkcwtynnkouofpfgrkzsrpwdgddngeetzennzsmwhdcmwfwlpalkylsfpalfoyahtpsotansgulftansfwlrhddaidcytkrkpscettlpvahljkaxdssgdifsgdfybknecahgkprdienbtayleeylldpttsnbtdwlktgsltkkhkaelkhtyktocpfmetcygdhsmkmsvlweatrsjymoglbthlltpagadihdcxrnadfewsspvycheeluldadrhlovyatvwrsghwkkeaspazshtjzclgobsbwbwatpttansgrhdcxehpryavolpeopdtdhkuyjzwpwfhkpsnncavdndfebavdkkwdsgdrndtiwnlemnieoybatpsotanshdhdcxpahtsbpmrlryvalttbdadnvsgdmteesgbatdykyndidwzswnbsjkfzlohsrpylstoyadtpsoiofeieinjyinjljtoycsfztpsotngdgmgwhflfaohdherpgorymspyyadpwsynpsonimfnlgckskhdtbqdjthnrffptluoecltuthlasnlgukepttnfwdsbypecfcahthhfspawemwtldyghcyaespbbaedyclnychmnrswebscxwdehuthfztettaolmhlndnskdaglfnenladkguhevoztbsdihdolcxinfmjkbgoyahtpsotansgulftansfwlrhddasoemflqzwszszestvspacmckhkpsmncnhnehssengahddpytbdlrwmfpwpbzihwegrldpkldspgsjnadwnpfemhpqzosdmdwctlsgdvykklrcptbpypyhyttlpfllfbywttdvohdcxrnadfewsspvycheeluldadrhlovyatvwrsghwkkeaspazshtjzclgobsbwbwatpttansgrhdcxlrspiyrnzcnsseisttvsnsvegetogrryfmampmimsponwknnamjtdezowerpwsdwoytpsoieiajzkpidtpsotanshdhdcxkelkotemrlolsfsbiadmhyrtfgchkshtoeptqdrnidmurnmelkrsswgysosrlkmnoyaxtpsotansghhdfzclvyspjzdicmimsgnefetygarpweneaareuefskprortyaknfpdtdtwktnckwywtmsjzcsfwsfgwrpotfnosaxololcwoxgufxseehotgtntheglrpmswmmtnlskherkahtddkln
│ {
│     ENCRYPTED [
│         'isA': "Edition"
│         {
│             'hasRecipient': SealedMessage
│         } [
│             'holder': XID(b15acbad)
│         ]
│         "club": XID(7c8ca337)
│         'hasRecipient': SealedMessage
│         'provenance': ProvenanceMark(a8700edf)
│     ]
│ } [
│     'signed': Signature
│ ]
│ 
│ ur:envelope/lftansfwlrhdfwwzurnnrntdtkrhjnsgwphlvlbzldmypdprbgfzfemoqzsaaytdrdwmwdkorsctsbtkdsfhfhyagllfndbzvdnsrpwywefehpdivecmdicekinbimcsnepkhejpfeksfrvefzgsfrutfhvdrfhlkekbahbkuopmgdlefefmsbgmkohhhghkylkbfxdnzcaomthddatansfphdcxenhtcybkcwtynnkouofpfgrkzsrpwdgddngeetzennzsmwhdcmwfwlpalkylsfpaoyamtpsotantkphddawldyaeadaectdakpkbglcswzhtwebztasevylkjsfwadsnolwnttyanbahdknllgpewfwdrnrofggogyao
│ ENCRYPTED [
│     'sskrShare': SSKRShare
│ ]
│ 
│ ur:envelope/lftansfwlrhdfwwzurnnrntdtkrhjnsgwphlvlbzldmypdprbgfzfemoqzsaaytdrdwmwdkorsctsbtkdsfhfhyagllfndbzvdnsrpwywefehpdivecmdicekinbimcsnepkhejpfeksfrvefzgsfrutfhvdrfhlkekbahbkuopmgdlefefmsbgmkohhhghkylkbfxdnzcaomthddatansfphdcxenhtcybkcwtynnkouofpfgrkzsrpwdgddngeetzennzsmwhdcmwfwlpalkylsfpaoyamtpsotantkphddawldyaeadadmtdskeqdpfbgjktddytdnymtpldyfrwmgrswiakopsamrlpezmpsolgteoiyrftagtfxnnjt
│ ENCRYPTED [
│     'sskrShare': SSKRShare
│ ]
│ 
│ ur:envelope/lftansfwlrhdfwwzurnnrntdtkrhjnsgwphlvlbzldmypdprbgfzfemoqzsaaytdrdwmwdkorsctsbtkdsfhfhyagllfndbzvdnsrpwywefehpdivecmdicekinbimcsnepkhejpfeksfrvefzgsfrutfhvdrfhlkekbahbkuopmgdlefefmsbgmkohhhghkylkbfxdnzcaomthddatansfphdcxenhtcybkcwtynnkouofpfgrkzsrpwdgddngeetzennzsmwhdcmwfwlpalkylsfpaoyamtpsotantkphddawldyaeadaocmcniozmptbnwmgygslahejllbwsvwbdmduyemvednctmngeldwfuyjoiswlrdknrodtdlbe
│ ENCRYPTED [
│     'sskrShare': SSKRShare
│ ]
│ 
```

## Verifying composed edition

No output indicates success.

```
clubs edition verify \
  --edition "$EDITION_UR" \
  --publisher "$PUBLISHER_XID"
```

## Extracting permit URs

List the sealed recipient messages to confirm the intended audience.

```
typeset -ga PERMIT_URS=("${(@f)$(clubs edition permits --edition "$EDITION_UR")%$'\n'}")
print -r -l -- "${PERMIT_URS[@]}"

│ ur:crypto-sealed/lftansfwlrhddaidcytkrkpscettlpvahljkaxdssgdifsgdfybknecahgkprdienbtayleeylldpttsnbtdwlktgsltkkhkaelkhtyktocpfmetcygdhsmkmsvlweatrsjymoglbthlltpagadihdcxrnadfewsspvycheeluldadrhlovyatvwrsghwkkeaspazshtjzclgobsbwbwatpttansgrhdcxehpryavolpeopdtdhkuyjzwpwfhkpsnncavdndfebavdkkwdsgdrndtiwnlemniecsiadeuo
│ ur:crypto-sealed/lftansfwlrhddasoemflqzwszszestvspacmckhkpsmncnhnehssengahddpytbdlrwmfpwpbzihwegrldpkldspgsjnadwnpfemhpqzosdmdwctlsgdvykklrcptbpypyhyttlpfllfbywttdvohdcxrnadfewsspvycheeluldadrhlovyatvwrsghwkkeaspazshtjzclgobsbwbwatpttansgrhdcxlrspiyrnzcnsseisttvsnsvegetogrryfmampmimsponwknnamjtdezowerpwsdwvytllgdw
```

## Decrypting content with Alice's permit

Try each permit against Alice's private keys until one works.

```
typeset -g PERMIT_CONTENT_UR=""
for permit in "${PERMIT_URS[@]}"; do
  PERMIT_OUTPUT=$(clubs content decrypt \
    --edition "$EDITION_UR" \
    --publisher "$PUBLISHER_XID" \
    --permit "$permit" \
    --identity "$ALICE_PRVKEYS" \
    --emit-ur 2>/dev/null)
  if [[ -n "$PERMIT_OUTPUT" ]]; then
    PERMIT_CONTENT_UR=${PERMIT_OUTPUT%%$'\n'*}
    echo "$PERMIT_CONTENT_UR"
    envelope format "$PERMIT_CONTENT_UR"
    break
  fi
done

│ ur:envelope/tpsplftpsokscehgihjziajljnihcxjyjlcxjyisihcxfljljpieinhsjtcxfxjzkpidcloytpsoihjyinjyjzihtpsojlflihjtihjkinjkcxfeieinjyinjljtialawspd
│ {
│     "Welcome to the Gordian Club!" [
│         "title": "Genesis Edition"
│     ]
│ }
```

## Decrypting content via SSKR shares

Show that a quorum of shares can recover the same plaintext without any permit.

```
SSKR_CONTENT_UR=$(clubs content decrypt \
  --edition "$EDITION_UR" \
  --publisher "$PUBLISHER_XID" \
  --sskr "${SSKR_URS[1]}" \
  --sskr "${SSKR_URS[2]}" \
  --emit-ur)
SSKR_CONTENT_UR=${SSKR_CONTENT_UR%%$'\n'*}
print -r -- "$SSKR_CONTENT_UR"
envelope format "$SSKR_CONTENT_UR"

│ ur:envelope/tpsplftpsokscehgihjziajljnihcxjyjlcxjyisihcxfljljpieinhsjtcxfxjzkpidcloytpsoihjyinjyjzihtpsojlflihjtihjkinjkcxfeieinjyinjljtialawspd
│ {
│     "Welcome to the Gordian Club!" [
│         "title": "Genesis Edition"
│     ]
│ }
```

## Authoring follow-up content envelope

Prepare the content for the club's second edition.

```
UPDATE_SUBJECT=$(envelope subject type string 'Club update: upcoming workshops and Q&A sessions')
echo "${UPDATE_SUBJECT}"
envelope format "${UPDATE_SUBJECT}"
echo ""
UPDATE_CLEAR=$(echo "${UPDATE_SUBJECT}" | envelope assertion add pred-obj string "title" string 'Second Edition')
echo "${UPDATE_CLEAR}"
envelope format "${UPDATE_CLEAR}"
echo ""
UPDATE_WRAPPED=$(envelope subject type wrapped "${UPDATE_CLEAR}")
echo "${UPDATE_WRAPPED}"
envelope format "${UPDATE_WRAPPED}"

│ ur:envelope/tpsoksdyfxjzkpidcxkpjoiehsjyihftcxkpjoiajljninjtiocxktjljpjejkisjljojkcxhsjtiecxgydsfpcxjkihjkjkinjljtjkkeecehtb
│ "Club update: upcoming workshops and Q&A sessions"
│ 
│ ur:envelope/lftpsoksdyfxjzkpidcxkpjoiehsjyihftcxkpjoiajljninjtiocxktjljpjejkisjljojkcxhsjtiecxgydsfpcxjkihjkjkinjljtjkoytpsoihjyinjyjzihtpsojtguihiajljtiecxfeieinjyinjljtsfctkbpk
│ "Club update: upcoming workshops and Q&A sessions" [
│     "title": "Second Edition"
│ ]
│ 
│ ur:envelope/tpsplftpsoksdyfxjzkpidcxkpjoiehsjyihftcxkpjoiajljninjtiocxktjljpjejkisjljojkcxhsjtiecxgydsfpcxjkihjkjkinjljtjkoytpsoihjyinjyjzihtpsojtguihiajljtiecxfeieinjyinjljtpdamptki
│ {
│     "Club update: upcoming workshops and Q&A sessions" [
│         "title": "Second Edition"
│     ]
│ }
```

## Capturing follow-up content digest

Record the new digest so the next provenance mark can attest to the update.

```
UPDATE_DIGEST=$(envelope digest "${UPDATE_WRAPPED}")
echo "${UPDATE_DIGEST}"

│ ur:digest/hdcxnlfleyrscltllfvsrncatbamasfyneryuyckwnhhsniowyemzmkotitkeofppkknjzoylyvy
```

## Advancing provenance mark chain

Issue the second mark (#1) and confirm its info digest matches the updated content.

```
SECOND_MARK=$(provenance next --comment "Second edition" --format ur --quiet --info "$UPDATE_DIGEST" demo/provenance-chain)
echo "$SECOND_MARK"
provenance print demo/provenance-chain --start 1 --end 1 --format markdown

│ ur:provenance/lfaohdhedpvybdhklscplfstfyskjtlbgtecjsolhhessbfhdyvtmkhdfpeywmyadmqzimgljpptoltdspvdhdhftefwchckfgsodrjydnhseoosqzeefhgtonhheeytnslywpcycfgycaurssaszcahkkdwcnksrsiakeeeidlpcsimctdyfwnswtcwgolokevdayksdnrsme
│ ---
│ 
│ 2025-10-01T09:17:39Z
│ 
│ #### ur:provenance/lfaohdhedpvybdhklscplfstfyskjtlbgtecjsolhhessbfhdyvtmkhdfpeywmyadmqzimgljpptoltdspvdhdhftefwchckfgsodrjydnhseoosqzeefhgtonhheeytnslywpcycfgycaurssaszcahkkdwcnksrsiakeeeidlpcsimctdyfwnswtcwgolokevdayksdnrsme
│ 
│ #### `🅟 UGLY ABLE ONYX DATA`
│ 
│ 🅟 🐹 😀 📦 👽
│ 
│ Second edition
│ 
```

## Composing second edition

Seal the follow-up content with the new mark and regenerate member permits.

```
SECOND_EDITION_RAW=$(clubs edition compose \
  --publisher "$PUBLISHER_XID" \
  --content "$UPDATE_WRAPPED" \
  --provenance "$SECOND_MARK" \
  --permit "$ALICE_XID" \
  --permit "$BOB_PUBKEYS" \
  --sskr 2of3)
print -r -- "$SECOND_EDITION_RAW"

│ ur:envelope/lftpsplntansfwlrhdgopmimcekbtovtswetcplgiywntdgapkqdfxwzftnyfpcmhefdskiobwidhndphkrpasplwfspemcmlehfknfhmyzsioynfeuyftgslnzewnjzehiegmktpslrcmspjnykrywllyeefeaaaejywphynbeopmaevlgajziozogwidgscfsnjkhnfzcweswzsahfwmkggdfpuopysoaovwpdztsagygamuzmcxwlhlhddatansfphdcxueaxeywluytbnymyqzhddwgrchkoihsfmdfnascyhsrklpvdkplolyzslbgsjevloyadtpsoiofeieinjyinjljtoyahtpsotansgulftansfwlrhddahhvosnrktbtajepdcfbtmnotwzssiyaxftksmemdcwdynlbsrsdwwsfttpfswmssveluzsdsrkgswsjzlfsrmokboesnrnknltzegduttejtoswttedrnnmkkigspllrgdnnlnhdcxcthgghnscxbnctjtpemecyfmolzmfswkemrylscxvykodpknbbgacsmsfezofmbdtansgrhdcxwydefmpygrcpwsbkqzkowsbkmokkdrgedytlkkaeotnbpdtapkimbdglkkwyisdloytpsoieiajzkpidtpsotanshdhdcxkelkotemrlolsfsbiadmhyrtfgchkshtoeptqdrnidmurnmelkrsswgysosrlkmnoycsfztpsotngdgmgwhflfaohdhedpvybdhklscplfstfyskjtlbgtecjsolhhessbfhdyvtmkhdfpeywmyadmqzimgljpptoltdspvdhdhftefwchckfgsodrjydnhseoosqzeefhgtonhheeytnslywpcycfgycaurssaszcahkkdwcnksrsiakeeeidlpcsimctdyfwnswtcwgolokevdaylfoyahtpsotansgulftansfwlrhddasnprpmuocpdpsndkytvdmkrymnhkbzhhdwhyndenpspmjzvwkkdmglspcxdyjttylgihdadwnegslblattlumsvtsppslnkbndgdgdndbzylnlptkgaagwwzgajtpmuoyklemhhdcxcthgghnscxbnctjtpemecyfmolzmfswkemrylscxvykodpknbbgacsmsfezofmbdtansgrhdcxvtsewyaskosegocacnmyuedsismyjziyvlkihflszsfhmdflrkjlsrltjkuywfdtoybatpsotanshdhdcxpahtsbpmrlryvalttbdadnvsgdmteesgbatdykyndidwzswnbsjkfzlohsrpylstoyaxtpsotansghhdfzdtpaykfnrsztoewplordleghdpfdemlydnrliycavldruoheryglbklklbiyltflehtnvaammsnbkpvsfmpereutcmdluytedmrlwzbklevslasahtsgwmhyhpprwmvovagrfehk
│ ur:envelope/lftansfwlrhdgopmimcekbtovtswetcplgiywntdgapkqdfxwzftnyfpcmhefdskiobwidhndphkrpasplwfspemcmlehfknfhmyzsioynfeuyftgslnzewnjzehiegmktpslrcmspjnykrywllyeefeaaaejywphynbeopmaevlgajziozogwidgscfsnjkhnfzcweswzsahfwmkggdfpuopysoaovwpdztsagygamuzmcxwlhlhddatansfphdcxueaxeywluytbnymyqzhddwgrchkoihsfmdfnascyhsrklpvdkplolyzslbgsjevloyamtpsotantkphddaldttaeadaejlbwfyhnjytyuefwcweeptlsgekibdjspmktytpstnchpkvscnmydwbwamemwzamfzhsbzny
│ ur:envelope/lftansfwlrhdgopmimcekbtovtswetcplgiywntdgapkqdfxwzftnyfpcmhefdskiobwidhndphkrpasplwfspemcmlehfknfhmyzsioynfeuyftgslnzewnjzehiegmktpslrcmspjnykrywllyeefeaaaejywphynbeopmaevlgajziozogwidgscfsnjkhnfzcweswzsahfwmkggdfpuopysoaovwpdztsagygamuzmcxwlhlhddatansfphdcxueaxeywluytbnymyqzhddwgrchkoihsfmdfnascyhsrklpvdkplolyzslbgsjevloyamtpsotantkphddaldttaeadadvwgrmthkyajodasfdylstyskhlndiaghuyjngmtplepfoebyoyzepsgwvsswynsrzolfsshs
│ ur:envelope/lftansfwlrhdgopmimcekbtovtswetcplgiywntdgapkqdfxwzftnyfpcmhefdskiobwidhndphkrpasplwfspemcmlehfknfhmyzsioynfeuyftgslnzewnjzehiegmktpslrcmspjnykrywllyeefeaaaejywphynbeopmaevlgajziozogwidgscfsnjkhnfzcweswzsahfwmkggdfpuopysoaovwpdztsagygamuzmcxwlhlhddatansfphdcxueaxeywluytbnymyqzhddwgrchkoihsfmdfnascyhsrklpvdkplolyzslbgsjevloyamtpsotantkphddaldttaeadaohnotzobgktlteofegtfpgubsiepkuyfrfpfxqzfyknfwrdadfnjnempysetozsmswkdifnwl
```

## Capturing second edition artifacts

Review the second-edition envelope and its fresh shard set before verification.

```
typeset -ga EDITION2_URS=("${(@f)${SECOND_EDITION_RAW%$'\n'}}")
EDITION2_UR=${EDITION2_URS[1]}
typeset -ga SSKR2_URS=("${EDITION2_URS[@]:1}")
for ur in "${EDITION2_URS[@]}"; do print -r -- "$ur"; envelope format "$ur"; echo ""; done

│ ur:envelope/lftpsplntansfwlrhdgopmimcekbtovtswetcplgiywntdgapkqdfxwzftnyfpcmhefdskiobwidhndphkrpasplwfspemcmlehfknfhmyzsioynfeuyftgslnzewnjzehiegmktpslrcmspjnykrywllyeefeaaaejywphynbeopmaevlgajziozogwidgscfsnjkhnfzcweswzsahfwmkggdfpuopysoaovwpdztsagygamuzmcxwlhlhddatansfphdcxueaxeywluytbnymyqzhddwgrchkoihsfmdfnascyhsrklpvdkplolyzslbgsjevloyadtpsoiofeieinjyinjljtoyahtpsotansgulftansfwlrhddahhvosnrktbtajepdcfbtmnotwzssiyaxftksmemdcwdynlbsrsdwwsfttpfswmssveluzsdsrkgswsjzlfsrmokboesnrnknltzegduttejtoswttedrnnmkkigspllrgdnnlnhdcxcthgghnscxbnctjtpemecyfmolzmfswkemrylscxvykodpknbbgacsmsfezofmbdtansgrhdcxwydefmpygrcpwsbkqzkowsbkmokkdrgedytlkkaeotnbpdtapkimbdglkkwyisdloytpsoieiajzkpidtpsotanshdhdcxkelkotemrlolsfsbiadmhyrtfgchkshtoeptqdrnidmurnmelkrsswgysosrlkmnoycsfztpsotngdgmgwhflfaohdhedpvybdhklscplfstfyskjtlbgtecjsolhhessbfhdyvtmkhdfpeywmyadmqzimgljpptoltdspvdhdhftefwchckfgsodrjydnhseoosqzeefhgtonhheeytnslywpcycfgycaurssaszcahkkdwcnksrsiakeeeidlpcsimctdyfwnswtcwgolokevdaylfoyahtpsotansgulftansfwlrhddasnprpmuocpdpsndkytvdmkrymnhkbzhhdwhyndenpspmjzvwkkdmglspcxdyjttylgihdadwnegslblattlumsvtsppslnkbndgdgdndbzylnlptkgaagwwzgajtpmuoyklemhhdcxcthgghnscxbnctjtpemecyfmolzmfswkemrylscxvykodpknbbgacsmsfezofmbdtansgrhdcxvtsewyaskosegocacnmyuedsismyjziyvlkihflszsfhmdflrkjlsrltjkuywfdtoybatpsotanshdhdcxpahtsbpmrlryvalttbdadnvsgdmteesgbatdykyndidwzswnbsjkfzlohsrpylstoyaxtpsotansghhdfzdtpaykfnrsztoewplordleghdpfdemlydnrliycavldruoheryglbklklbiyltflehtnvaammsnbkpvsfmpereutcmdluytedmrlwzbklevslasahtsgwmhyhpprwmvovagrfehk
│ {
│     ENCRYPTED [
│         'isA': "Edition"
│         {
│             'hasRecipient': SealedMessage
│         } [
│             'holder': XID(b15acbad)
│         ]
│         "club": XID(7c8ca337)
│         'hasRecipient': SealedMessage
│         'provenance': ProvenanceMark(db00a425)
│     ]
│ } [
│     'signed': Signature
│ ]
│ 
│ ur:envelope/lftansfwlrhdgopmimcekbtovtswetcplgiywntdgapkqdfxwzftnyfpcmhefdskiobwidhndphkrpasplwfspemcmlehfknfhmyzsioynfeuyftgslnzewnjzehiegmktpslrcmspjnykrywllyeefeaaaejywphynbeopmaevlgajziozogwidgscfsnjkhnfzcweswzsahfwmkggdfpuopysoaovwpdztsagygamuzmcxwlhlhddatansfphdcxueaxeywluytbnymyqzhddwgrchkoihsfmdfnascyhsrklpvdkplolyzslbgsjevloyamtpsotantkphddaldttaeadaejlbwfyhnjytyuefwcweeptlsgekibdjspmktytpstnchpkvscnmydwbwamemwzamfzhsbzny
│ ENCRYPTED [
│     'sskrShare': SSKRShare
│ ]
│ 
│ ur:envelope/lftansfwlrhdgopmimcekbtovtswetcplgiywntdgapkqdfxwzftnyfpcmhefdskiobwidhndphkrpasplwfspemcmlehfknfhmyzsioynfeuyftgslnzewnjzehiegmktpslrcmspjnykrywllyeefeaaaejywphynbeopmaevlgajziozogwidgscfsnjkhnfzcweswzsahfwmkggdfpuopysoaovwpdztsagygamuzmcxwlhlhddatansfphdcxueaxeywluytbnymyqzhddwgrchkoihsfmdfnascyhsrklpvdkplolyzslbgsjevloyamtpsotantkphddaldttaeadadvwgrmthkyajodasfdylstyskhlndiaghuyjngmtplepfoebyoyzepsgwvsswynsrzolfsshs
│ ENCRYPTED [
│     'sskrShare': SSKRShare
│ ]
│ 
│ ur:envelope/lftansfwlrhdgopmimcekbtovtswetcplgiywntdgapkqdfxwzftnyfpcmhefdskiobwidhndphkrpasplwfspemcmlehfknfhmyzsioynfeuyftgslnzewnjzehiegmktpslrcmspjnykrywllyeefeaaaejywphynbeopmaevlgajziozogwidgscfsnjkhnfzcweswzsahfwmkggdfpuopysoaovwpdztsagygamuzmcxwlhlhddatansfphdcxueaxeywluytbnymyqzhddwgrchkoihsfmdfnascyhsrklpvdkplolyzslbgsjevloyamtpsotantkphddaldttaeadaohnotzobgktlteofegtfpgubsiepkuyfrfpfxqzfyknfwrdadfnjnempysetozsmswkdifnwl
│ ENCRYPTED [
│     'sskrShare': SSKRShare
│ ]
│ 
```

## Verifying second edition

No output indicates success.

```
clubs edition verify \
  --edition "$EDITION2_UR" \
  --publisher "$PUBLISHER_XID"
```

## Validating edition sequence

Confirm both editions belong to the same club and form a continuous chain. Editions may be provided in any order. No output indicates success.

```
clubs edition sequence \
  --edition "$EDITION2_UR" \
  --edition "$EDITION_UR"
```

