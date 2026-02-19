# 0x54 / ILED-Color protocol specification
This document describes the ILED protocol being sent over Bluetooth Attribute Protocol (as a serialized value).  
It uses write_without_response from client to server (us to device), and handle_value_notification from server to client (device to us).  
The use of write_without_response and notifications makes the communication async, meaning it's possible for things to be sent and received out-of-order.  
Furthermore it doesn't implement message id's (except for chunks) so state has to be kept client side if desired.  
  
It is not official documentation, it is an attempt at documenting it through sniffing and experimenting, so expect missing fields and undefined behaviour.  

---
## Common fields
All command and command-notification packets take the form

|Field|Bytes|
|:-----|:-----|
|Protocol marker (0x54) |1|
|Command Type           |1|
|pack length            |2|
|**command specific field**      |?|
|..                     |..|
|**command specific field**      |?|
|checksum               |2|

### Protocol marker
All packets start with the byte `0x54`.
### Command type
Single byte denoting the type of command or command-notification.
See *Command types by id*.
### Pack length
Number of bytes following this field, in this packet, to its end, including the end checksum.
### Command specific field(s)
One or more fields of varying type and size - specific to the command type - that contain the actual command data.
See *Command types by id*.
### Checksum
checksum is a 16-bit running bytewise sum of the whole packet, including other checksum and length fields, but by necessity excluding itself.

---

## Command types by id

### Continue 0x00
|Field|Bytes|
|:-----|:-----|
|0x0            |3|
|split pack num |1|
|sub data len   |2|
|data chksum    |4|
|data           |?|

#### Data
##### Single image mode
0E:0E 0x1  
0F:25 0x0  
26:27 HR  
28:29 VR  
2A:2B 0x0  
2C:2D 0x0001  
2E:2F 0x0001  
30:31 0x0001  
32:32 0x32  
33:33 0x0  
34:34 0x64  
35:37 0x0  
RGB color  

#### Continue notification
|Field|Bytes|
|:-----|:-----|
|0x0            |3|
|split pack num |1|
|0x01           |1|

---

### EndStream 0x01
|Field|Bytes|
|:-----|:-----|
|0x1            |1|

#### EndStream notification

---

### StartStream 0x06
|Field|Bytes|
|:-----|:-----|
|data chksum    |4|
|0x0            |2|
|total data len |2|
|0x0            |3|

#### StartStream notification
|Field|Bytes|
|:-----|:-----|
|exp pack cnt   |1|

---

### Dimming 0x09
|Field|Bytes|
|:-----|:-----|
|Dimming        |1|
|0x00 padding   |8|

##### Dimming
|Value|Meaning|
|:-----|:-----|
|0x00|Brightest|
|0x01|Brightest|
|0x02|Brighter|
|..|..|
|0x09|Dimmer|
|0x0A|Dimmest|
|0x0B - FF|Brightest|

#### Dimming notification
|Field|Bytes|
|:-----|:-----|

---

### Display Enable 0x0A
|Field|Bytes|
|:-----|:-----|
|Enable         |1|
|0x00 padding   |8|

##### Enable
|Value|Result|
|:-----|:-----|
|0x00  |Off|
|0x01 - FF|On|

#### Display Enable Notification
|Field|Bytes|
|:-----|:-----|

---

### Connect 0x0D
|Field|Bytes|
|:-----|:-----|
|0x00|1|

#### Connect notification
|Field|Bytes|
|:-----|:-----|
|0x00|2|

---

### Password operations 0x0E
|Field|Bytes|
|:-----|:-----|
|OpCode         |1|
|oldPass null=0 |6|
|newPass        |6|

##### OpCode
|Value|Meaning|
|:-----|:-----|
|0x00|set|
|0x01|change|
|0x02|unset|

#### Password operations notification
|Field|Bytes|
|:-----|:-----|
|PassOP response |1|

##### PassOp response
|Value|Meaning|
|:-----|:-----|
|0x01|success|
|0x02|failure|

---

### check pass 0x0F
|Field|Bytes|
|:-----|:-----|
|pass           |6|

#### check pass notification
|Field|Bytes|
|:-----|:-----|
|PassCheck response |1|

##### PassCheck response
|Value|Meaning|
|:-----|:-----|
|0x01|correct|
|0x02|incorrect|
|0x03|unset|

--- 
## Early connection packets

    write 0x0082
    0066320db73158a35a255d051758e95ed4
    recieve 0x0084
    018eb91d1ea78352be80f3f5d428853aa8

    write 0x0082  
    0270617373
    recieve 0x0084  
    00ea770535d4ef0a6ae7a2eaac7958145d

    write 0x0082  
    01567b6736227208d4be4e4ebc779d744f  
    recieve 0x0084  
    0270617373  

    write 0x0082  
    fedcbac003000602ffffffff00ef  
    recieve 0x0084  
    fedcba0003003e000202002005010000000009029e193df98b8c0e0006040000004e0002050003080100020900020a00020601050d0080021c0811009e193d7c21be021300ef  


---
