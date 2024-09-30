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

| Field                       | Type     | Content         | Expected |
| --------------------------- | -------- | --------------- | -------- |
| FrostSigningPackage Len     | bytes... | Message to Sign |          |
| FrostSigningPackage Content | bytes... | Message to Sign |          |
| Tx Randomizer Len           | bytes... | Message to Sign |          |
| Tx Randomizer Content       | bytes... | Message to Sign |          |
| Tx Hash                     | bytes... | Message to Sign |          |

#### Response

| Field   | Type     | Content                        | Note                     |
| ------- | -------- | ------------------------------ | ------------------------ |
| CHUNKS  | byte (1) | Chunks of data to be retrieved |                          |
| SW1-SW2 | byte (2) | Return code                    | see list of return codes |
