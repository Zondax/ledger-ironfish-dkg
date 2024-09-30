# Ironfish App

## General structure

The general structure of commands and responses is as follows:

### Commands

| Field   | Type     | Content                | Note |
| :------ | :------- | :--------------------- | ---- |
| CLA     | byte (1) | Application Identifier | 0x63 |
| INS     | byte (1) | Instruction ID         |      |
| P1      | byte (1) | Parameter 1            |      |
| P2      | byte (1) | Parameter 2            |      |
| L       | byte (1) | Bytes in payload       |      |
| PAYLOAD | byte (L) | Payload                |      |

### Response

| Field   | Type     | Content     | Note                     |
| ------- | -------- | ----------- | ------------------------ |
| ANSWER  | byte (?) | Answer      | depends on the command   |
| SW1-SW2 | byte (2) | Return code | see list of return codes |

### Return codes

| Return code | Description              |
| ----------- | ------------------------ |
| 0x6985      | Deny                     |
| 0x6A86      | Wrong P1/P2              |
| 0x6D00      | INS not supported        |
| 0x6E00      | CLA not supported        |
| 0xB001      | Tx display fail          |
| 0xB002      | Addr display fail        |
| 0xB004      | Tx wrong length          |
| 0xB005      | Tx parsing fail          |
| 0xB006      | Tx hash fail             |
| 0xB008      | Tx sign fail             |
| 0xB009      | Key derive fail          |
| 0xB00A      | Version parsing fail     |
| 0xB00B      | Dkg round 2 fail         |
| 0xB00C      | Dkg round 3 fail         |
| 0xB00D      | Invalid key type         |
| 0xB00E      | Invalid identity         |
| 0xB00F      | Invalid payload          |
| 0xB010      | Buffer out of bounds     |
| 0xB011      | Invalid signing package  |
| 0xB012      | Invalid randomizer       |
| 0xB013      | Invalid signing nonces   |
| 0xB014      | Invalid identity index   |
| 0xB015      | Invalid key package      |
| 0xB016      | Invalid public package   |
| 0xB017      | Invalid group secret key |
| 0xB018      | Invalid scalar           |
| 0xB019      | Decryption fail          |
| 0xB020      | Encryption fail          |
| 0xB021      | Invalid NVM write        |
| 0xB022      | Invalid Dkg status       |
| 0xB023      | Invalid Dkg keys version |
| 0xB024      | Too many participants    |
| 0xB025      | Invalid Tx hash          |
| 0x9000      | Success                  |

---

## Command definition

### GET_VERSION

#### Command

| Field | Type     | Content                | Expected |
| ----- | -------- | ---------------------- | -------- |
| CLA   | byte (1) | Application Identifier | 0x63     |
| INS   | byte (1) | Instruction ID         | 0x00     |
| P1    | byte (1) | Parameter 1            | ignored  |
| P2    | byte (1) | Parameter 2            | ignored  |
| L     | byte (1) | Bytes in payload       | 0        |

#### Response

| Field   | Type     | Content          | Note                            |
| ------- | -------- | ---------------- | ------------------------------- |
| TEST    | byte (1) | Test Mode        | 0xFF means test mode is enabled |
| MAJOR   | byte (2) | Version Major    | 0..65535                        |
| MINOR   | byte (2) | Version Minor    | 0..65535                        |
| PATCH   | byte (2) | Version Patch    | 0..65535                        |
| LOCKED  | byte (1) | Device is locked |                                 |
| SW1-SW2 | byte (2) | Return code      | see list of return codes        |

---

### INS_DKG_GET_IDENTITY

#### Command

| Field | Type     | Content                   | Expected |
| ----- | -------- | ------------------------- | -------- |
| CLA   | byte (1) | Application Identifier    | 0x63     |
| INS   | byte (1) | Instruction ID            | 0x10     |
| P1    | byte (1) | Request User confirmation | No = 0   |
| P2    | byte (1) | KeyType                   | 0 ~ 3    |
| L     | byte (1) | Bytes in payload          | 0        |

#### Response

| Field        | Type       | Content      | Note                     |
| ------------ | ---------- | ------------ | ------------------------ |
| DKG Identity | byte (129) | DKG Identity | The derived DKG identity |
| SW1-SW2      | byte (2)   | Return code  | see list of return codes |

---

### INS_DKG_GET_KEYS

#### Command

| Field | Type     | Content                   | Expected |
| ----- | -------- | ------------------------- | -------- |
| CLA   | byte (1) | Application Identifier    | 0x63     |
| INS   | byte (1) | Instruction ID            | 0x16     |
| P1    | byte (1) | Request User confirmation | No = 0   |
| P2    | byte (1) | ---                       | not used |
| L     | byte (1) | Bytes in payload          | 1        |
| Index | byte (1) | Identity to derive        | 0 ~ 5    |

#### Response

If the input parameters are wrong or some error happens during the computation of this command, the app will return only the error code.

**|---> For KeyType = 0 (PublicAddress)**

| Field   | Type      | Content        | Note                     |
| ------- | --------- | -------------- | ------------------------ |
| Address | byte (32) | Public address |                          |
| SW1-SW2 | byte (2)  | Return code    | see list of return codes |

---

**|---> For KeyType = 1 (ViewKey)**

| Field   | Type      | Content            | Note                     |
| ------- | --------- | ------------------ | ------------------------ |
| ViewKey | byte (64) | ViewKey            | {ak, nk}                 |
| IVK     | byte (32) | IncomingViewingKey | {ivk}                    |
| OVK     | byte (32) | OutgoingViewingKey | {ovk}                    |
| SW1-SW2 | byte (2)  | Return code        | see list of return codes |

---

**|---> For KeyType = 2 (ProofGenerationKey)**

| Field               | Type      | Content            | Note                     |
| ------------------- | --------- | ------------------ | ------------------------ |
| ProofGeneration key | byte (64) | ProofGenerationKey | {ak, nsk}                |
| SW1-SW2             | byte (2)  | Return code        | see list of return codes |

---

**|---> For KeyType = 3 (Current DKG Identity)**

| Field        | Type       | Content      | Note                                                   |
| ------------ | ---------- | ------------ | ------------------------------------------------------ |
| DKG Identity | byte (129) | DKG Identity | The DKG identity used for the current multisig account |
| SW1-SW2      | byte (2)   | Return code  | see list of return codes                               |

---

### INS_DKG_SIGN

#### Command

| Field | Type     | Content                | Expected  |
| ----- | -------- | ---------------------- | --------- |
| CLA   | byte (1) | Application Identifier | 0x63      |
| INS   | byte (1) | Instruction ID         | 0x15      |
| P1    | byte (1) | Payload desc           | 0 = init  |
|       |          |                        | 1 = add   |
|       |          |                        | 2 = last  |
| P2    | byte (1) | ----                   | not used  |
| L     | byte (1) | Bytes in payload       | (depends) |

The first packet/chunk includes only the derivation path

All other packets/chunks contain data chunks that are described below

##### First Packet

| Field   | Type     | Content              | Expected |
| ------- | -------- | -------------------- | -------- |
| Path[0] | byte (4) | Derivation Path Data | 44       |
| Path[1] | byte (4) | Derivation Path Data | 434      |
| Path[2] | byte (4) | Derivation Path Data | ?        |
| Path[3] | byte (4) | Derivation Path Data | ?        |
| Path[4] | byte (4) | Derivation Path Data | ?        |

##### Other Chunks/Packets

| Field                       | Type      | Content                               | Expected |
| --------------------------- | --------- | ------------------------------------- | -------- |
| TxRandomizer Len            | byte (2)  | Tx randomizer value len (u16 be)      |          |
| TxRandomizer Content        | bytes...  | Tx randomizer value                   |          |
| FrostSigningPackage Len     | byte (2)  | Tx frost signing package len (u16 be) |          |
| FrostSigningPackage Content | bytes...  | Tx frost signing package              |          |
| Tx Hash                     | byte (32) | Tx Hash                               |          |

#### Response

| Field   | Type     | Content                        | Note                     |
| ------- | -------- | ------------------------------ | ------------------------ |
| CHUNKS  | byte (1) | Chunks of data to be retrieved |                          |
| SW1-SW2 | byte (2) | Return code                    | see list of return codes |

---

### INS_DKG_ROUND_1

#### Command

| Field | Type     | Content                | Expected  |
| ----- | -------- | ---------------------- | --------- |
| CLA   | byte (1) | Application Identifier | 0x63      |
| INS   | byte (1) | Instruction ID         | 0x11      |
| P1    | byte (1) | Payload desc           | 0 = init  |
|       |          |                        | 1 = add   |
|       |          |                        | 2 = last  |
| P2    | byte (1) | ----                   | not used  |
| L     | byte (1) | Bytes in payload       | (depends) |

The first packet/chunk includes only the derivation path

All other packets/chunks contain data chunks that are described below

##### First Packet

| Field   | Type     | Content              | Expected |
| ------- | -------- | -------------------- | -------- |
| Path[0] | byte (4) | Derivation Path Data | 44       |
| Path[1] | byte (4) | Derivation Path Data | 434      |
| Path[2] | byte (4) | Derivation Path Data | ?        |
| Path[3] | byte (4) | Derivation Path Data | ?        |
| Path[4] | byte (4) | Derivation Path Data | ?        |

##### Other Chunks/Packets

| Field               | Type     | Content                                    | Expected |
| ------------------- | -------- | ------------------------------------------ | -------- |
| Identity Index      | byte (1) | DKG Identities Index to derive secret from |          |
| Identities Elements | byte (1) | Identities qty (u8)                        |          |
| Identities          | bytes... | Identities involved on the DKG process     |          |
| Min Signers         | byte (1) | Minimum signers for the DKG process        |          |

#### Response

| Field   | Type     | Content                        | Note                     |
| ------- | -------- | ------------------------------ | ------------------------ |
| CHUNKS  | byte (1) | Chunks of data to be retrieved |                          |
| SW1-SW2 | byte (2) | Return code                    | see list of return codes |

---

### INS_DKG_ROUND_2

#### Command

| Field | Type     | Content                | Expected  |
| ----- | -------- | ---------------------- | --------- |
| CLA   | byte (1) | Application Identifier | 0x63      |
| INS   | byte (1) | Instruction ID         | 0x12      |
| P1    | byte (1) | Payload desc           | 0 = init  |
|       |          |                        | 1 = add   |
|       |          |                        | 2 = last  |
| P2    | byte (1) | ----                   | not used  |
| L     | byte (1) | Bytes in payload       | (depends) |

The first packet/chunk includes only the derivation path

All other packets/chunks contain data chunks that are described below

##### First Packet

| Field   | Type     | Content              | Expected |
| ------- | -------- | -------------------- | -------- |
| Path[0] | byte (4) | Derivation Path Data | 44       |
| Path[1] | byte (4) | Derivation Path Data | 434      |
| Path[2] | byte (4) | Derivation Path Data | ?        |
| Path[3] | byte (4) | Derivation Path Data | ?        |
| Path[4] | byte (4) | Derivation Path Data | ?        |

##### Other Chunks/Packets

| Field                      | Type     | Content                                                 | Expected  |
| -------------------------- | -------- | ------------------------------------------------------- | --------- |
| Identity Index             | byte (1) | DKG Identities Index to derive secret from              | (depends) |
| Round 1 PP Qty             | byte (1) | Qty of round 1 public packages (u8)                     | (depends) |
| Round 1 Package Len        | byte (1) | Length of each public package                           | (depends) |
| Round 1 Packages           | bytes... | Round 1 public package from participants (concatenated) | (depends) |
| Round 1 Secret Package Len | byte (1) | Length of the secret package                            | (depends) |
| Round 1 Secret Package     | bytes... | Secret pakcage generated on round 1                     | (depends) |

#### Response

| Field   | Type     | Content                        | Note                     |
| ------- | -------- | ------------------------------ | ------------------------ |
| CHUNKS  | byte (1) | Chunks of data to be retrieved |                          |
| SW1-SW2 | byte (2) | Return code                    | see list of return codes |

---

### INS_DKG_ROUND_3

#### Command

| Field | Type     | Content                | Expected  |
| ----- | -------- | ---------------------- | --------- |
| CLA   | byte (1) | Application Identifier | 0x63      |
| INS   | byte (1) | Instruction ID         | 0x15      |
| P1    | byte (1) | Payload desc           | 0 = init  |
|       |          |                        | 1 = add   |
|       |          |                        | 2 = last  |
| P2    | byte (1) | ----                   | not used  |
| L     | byte (1) | Bytes in payload       | (depends) |

The first packet/chunk includes only the derivation path

All other packets/chunks contain data chunks that are described below

##### First Packet

| Field   | Type     | Content              | Expected |
| ------- | -------- | -------------------- | -------- |
| Path[0] | byte (4) | Derivation Path Data | 44       |
| Path[1] | byte (4) | Derivation Path Data | 434      |
| Path[2] | byte (4) | Derivation Path Data | ?        |
| Path[3] | byte (4) | Derivation Path Data | ?        |
| Path[4] | byte (4) | Derivation Path Data | ?        |

##### Other Chunks/Packets

| Field                      | Type     | Content                                                 | Expected  |
| -------------------------- | -------- | ------------------------------------------------------- | --------- |
| Identity Index             | byte (1) | DKG Identities Index to derive secret from              | (depends) |
| Round 1 PP Qty             | byte (1) | Qty of round 1 public packages (u8)                     | (depends) |
| Round 1 Package Len        | byte (1) | Length of each public package                           | (depends) |
| Round 1 Packages           | bytes... | Round 1 public package from participants (concatenated) | (depends) |
| Round 2 PP Qty             | byte (1) | Qty of round 2 public packages (u8)                     | (depends) |
| Round 2 Package Len        | byte (1) | Length of each public package                           | (depends) |
| Round 2 Packages           | bytes... | Round 2 public package from participants (concatenated) | (depends) |
| Round 2 Secret Package Len | byte (1) | Length of the secret package                            | (depends) |
| Round 2 Secret Package     | bytes... | Secret pakcage generated on round 2                     | (depends) |
| Identities Qty             | byte (1) | Identities qty (u8)                                     | (depends) |
| Identities                 | bytes... | Identities involved on the DKG process                  | (depends) |
| GSK Qty                    | byte (1) | Qty of group shared keys (u8)                           | (depends) |
| GSK Len                    | byte (1) | Length of each group shared keys                        | (depends) |
| GSK                        | bytes... | Group shared keys from participants (concatenated)      | (depends) |

#### Response

| Field   | Type     | Content                        | Note                     |
| ------- | -------- | ------------------------------ | ------------------------ |
| CHUNKS  | byte (1) | Chunks of data to be retrieved |                          |
| SW1-SW2 | byte (2) | Return code                    | see list of return codes |

---

### INS_DKG_COMMITMETS

#### Command

| Field | Type     | Content                | Expected  |
| ----- | -------- | ---------------------- | --------- |
| CLA   | byte (1) | Application Identifier | 0x63      |
| INS   | byte (1) | Instruction ID         | 0x14      |
| P1    | byte (1) | Payload desc           | 0 = init  |
|       |          |                        | 1 = add   |
|       |          |                        | 2 = last  |
| P2    | byte (1) | ----                   | not used  |
| L     | byte (1) | Bytes in payload       | (depends) |

The first packet/chunk includes only the derivation path

All other packets/chunks contain data chunks that are described below

##### First Packet

| Field   | Type     | Content              | Expected |
| ------- | -------- | -------------------- | -------- |
| Path[0] | byte (4) | Derivation Path Data | 44       |
| Path[1] | byte (4) | Derivation Path Data | 434      |
| Path[2] | byte (4) | Derivation Path Data | ?        |
| Path[3] | byte (4) | Derivation Path Data | ?        |
| Path[4] | byte (4) | Derivation Path Data | ?        |

##### Other Chunks/Packets

| Field   | Type      | Content                            | Expected  |
| ------- | --------- | ---------------------------------- | --------- |
| Tx Hash | byte (32) | Tx hash to calculated commitmet to | (depends) |

#### Response

| Field   | Type     | Content                        | Note                     |
| ------- | -------- | ------------------------------ | ------------------------ |
| CHUNKS  | byte (1) | Chunks of data to be retrieved |                          |
| SW1-SW2 | byte (2) | Return code                    | see list of return codes |

---

### INS_DKG_GET_IDENTITIES

#### Command

| Field | Type     | Content                | Expected |
| ----- | -------- | ---------------------- | -------- |
| CLA   | byte (1) | Application Identifier | 0x63     |
| INS   | byte (1) | Instruction ID         | 0x17     |
| P1    | byte (1) | Parameter 1            | ignored  |
| P2    | byte (1) | Parameter 2            | ignored  |
| L     | byte (1) | Bytes in payload       | 0        |

#### Response

| Field   | Type     | Content                        | Note                     |
| ------- | -------- | ------------------------------ | ------------------------ |
| CHUNKS  | byte (1) | Chunks of data to be retrieved |                          |
| SW1-SW2 | byte (2) | Return code                    | see list of return codes |

---

### INS_DKG_GET_PUBLIC_PACKAGE

#### Command

| Field | Type     | Content                | Expected |
| ----- | -------- | ---------------------- | -------- |
| CLA   | byte (1) | Application Identifier | 0x63     |
| INS   | byte (1) | Instruction ID         | 0x18     |
| P1    | byte (1) | Parameter 1            | ignored  |
| P2    | byte (1) | Parameter 2            | ignored  |
| L     | byte (1) | Bytes in payload       | 0        |

#### Response

| Field   | Type     | Content                        | Note                     |
| ------- | -------- | ------------------------------ | ------------------------ |
| CHUNKS  | byte (1) | Chunks of data to be retrieved |                          |
| SW1-SW2 | byte (2) | Return code                    | see list of return codes |

---

---

### INS_DKG_BACKUP_KEYS

#### Command

| Field | Type     | Content                | Expected |
| ----- | -------- | ---------------------- | -------- |
| CLA   | byte (1) | Application Identifier | 0x63     |
| INS   | byte (1) | Instruction ID         | 0x19     |
| P1    | byte (1) | Parameter 1            | ignored  |
| P2    | byte (1) | Parameter 2            | ignored  |
| L     | byte (1) | Bytes in payload       | 0        |

#### Response

| Field   | Type     | Content                        | Note                     |
| ------- | -------- | ------------------------------ | ------------------------ |
| CHUNKS  | byte (1) | Chunks of data to be retrieved |                          |
| SW1-SW2 | byte (2) | Return code                    | see list of return codes |

---

### INS_DKG_RESTORE_KEYS

#### Command

| Field | Type     | Content                | Expected  |
| ----- | -------- | ---------------------- | --------- |
| CLA   | byte (1) | Application Identifier | 0x63      |
| INS   | byte (1) | Instruction ID         | 0x1a      |
| P1    | byte (1) | Payload desc           | 0 = init  |
|       |          |                        | 1 = add   |
|       |          |                        | 2 = last  |
| P2    | byte (1) | ----                   | not used  |
| L     | byte (1) | Bytes in payload       | (depends) |

The first packet/chunk includes only the derivation path

All other packets/chunks contain data chunks that are described below

##### First Packet

| Field   | Type     | Content              | Expected |
| ------- | -------- | -------------------- | -------- |
| Path[0] | byte (4) | Derivation Path Data | 44       |
| Path[1] | byte (4) | Derivation Path Data | 434      |
| Path[2] | byte (4) | Derivation Path Data | ?        |
| Path[3] | byte (4) | Derivation Path Data | ?        |
| Path[4] | byte (4) | Derivation Path Data | ?        |

##### Other Chunks/Packets

| Field            | Type     | Content                                 | Expected  |
| ---------------- | -------- | --------------------------------------- | --------- |
| Encrypted Backup | bytes... | Encrypted data from backup keys command | (depends) |

#### Response

| Field   | Type     | Content     | Note                     |
| ------- | -------- | ----------- | ------------------------ |
| SW1-SW2 | byte (2) | Return code | see list of return codes |

---

### INS_REVIEW_TX

#### Command

| Field | Type     | Content                | Expected  |
| ----- | -------- | ---------------------- | --------- |
| CLA   | byte (1) | Application Identifier | 0x63      |
| INS   | byte (1) | Instruction ID         | 0x1c      |
| P1    | byte (1) | Payload desc           | 0 = init  |
|       |          |                        | 1 = add   |
|       |          |                        | 2 = last  |
| P2    | byte (1) | ----                   | not used  |
| L     | byte (1) | Bytes in payload       | (depends) |

The first packet/chunk includes only the derivation path

All other packets/chunks contain data chunks that are described below

##### First Packet

| Field   | Type     | Content              | Expected |
| ------- | -------- | -------------------- | -------- |
| Path[0] | byte (4) | Derivation Path Data | 44       |
| Path[1] | byte (4) | Derivation Path Data | 434      |
| Path[2] | byte (4) | Derivation Path Data | ?        |
| Path[3] | byte (4) | Derivation Path Data | ?        |
| Path[4] | byte (4) | Derivation Path Data | ?        |

##### Other Chunks/Packets

| Field         | Type     | Content                    | Expected  |
| ------------- | -------- | -------------------------- | --------- |
| Serialized Tx | bytes... | Serialized tx to be signed | (depends) |

#### Response

| Field   | Type     | Content                        | Note                     |
| ------- | -------- | ------------------------------ | ------------------------ |
| CHUNKS  | byte (1) | Chunks of data to be retrieved |                          |
| SW1-SW2 | byte (2) | Return code                    | see list of return codes |

---

### INS_GET_RESULT

#### Command

| Field | Type     | Content                | Expected |
| ----- | -------- | ---------------------- | -------- |
| CLA   | byte (1) | Application Identifier | 0x63     |
| INS   | byte (1) | Instruction ID         | 0x1b     |
| P1    | byte (1) | Chunk index to fetch   | 0..255   |
| P2    | byte (1) | ----                   | not used |
| L     | byte (0) | Bytes in payload       | not used |

Some instructions save the result on flash once executed. The response on those cases is the amount of pages/chunks (data len / 253) to fetch. This command allows to fetch those pages/chunks.

#### Response

| Field   | Type     | Content                 | Note                     |
| ------- | -------- | ----------------------- | ------------------------ |
| CHUNK   | bytes... | Chunk of data retrieved |                          |
| SW1-SW2 | byte (2) | Return code             | see list of return codes |

---
