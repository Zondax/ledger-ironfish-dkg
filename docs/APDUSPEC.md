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

| Return code | Description             |
| ----------- | ----------------------- |
| 0x6985      | Deny                    |
| 0x6A86      | Wrong P1/P2             |
| 0x6D00      | INS not supported       |
| 0x6E00      | CLA not supported       |
| 0xB001      | Tx display fail         |
| 0xB002      | Addr display fail       |
| 0xB004      | Tx wrong length         |
| 0xB005      | Tx parsing fail         |
| 0xB006      | Tx hash fail            |
| 0xB008      | Tx sign fail            |
| 0xB009      | Key derive fail         |
| 0xB00A      | Version parsing fail    |
| 0xB00B      | Dkg round 2 fail        |
| 0xB00C      | Dkg round 3 fail        |
| 0xB00D      | Invalid key type        |
| 0xB00E      | Invalid identity        |
| 0xB00F      | Invalid payload         |
| 0xB010      | Buffer out of bounds    |
| 0xB011      | Invalid signing package |
| 0xB012      | Invalid randomizer      |
| 0xB013      | Invalid signing nonces  |
| 0xB014      | Invalid identity index  |
| 0xB015      | Invalid key package     |
| 0xB016      | Invalid public package  |
| 0xB017      | Invalid group secret key|
| 0xB018      | Invalid scalar          |
| 0xB019      | Decryption fail         |
| 0xB020      | Encryption fail         |
| 0xB021      | Invalid NVM write       |
| 0xB022      | Invalid Dkg status      |
| 0xB023      | Invalid Dkg keys version|
| 0xB024      | Too many participants   |
| 0xB025      | Invalid Tx hash         |
| 0x9000      | Success                 |

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
