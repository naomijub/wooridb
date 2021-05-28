## RFC - brainstorm for now

## Types:
```rust
pub enum Types {
    Char(char),
    Integer(isize),
    String(String),
    Uuid(Uuid),
    Float(f64),
    Boolean(bool),
    Vector(Vec<Types>),
    Map(HashMap<String, Types>),
    Hash(String),
    Precise(String),
    DateTime(DateTime<Utc>),
    Nil,
}
```

```rust
message KeyPair {
  string key = 1;
  WqpTypes value = 2;
}

message WqlTypes {
  oneof types {
    bytes char = 1;
    int64 int = 2;
    string string = 3;
    string uuid = 4;
    double float = 5;
    bool boolean = 6;
    // Vector(Vec<Types>) = 7; cyclic?
    repeated WqpTypes vec = 7;
    // Map(HashMap<String, Types>) = 8; cyclic?? or Map like in Order
    repeat KeyPair map = 8;
    string precise = 9;
    string datetime = 10; // or datetime
    Empty nil = 11;
  }
}
```

## Requests

### TX/Query
tx/query: String

```rust
syntax "proto3";

message Tx {
  string tx = 1;
}

message Query {
  string query = 1;
}
```

### EntityHistory
```rust
EntityHistory {
  entity_key: String,
  entity_id: Uuid,
  start_datetime: Option<DateTime<Utc>>,
  end_datetime: Option<DateTime<Utc>>,
}
```

```rust


message EntityHistory {
    string entity_id = 1;
    string entity_key = 2;
    // or `StringValue` which proto?. import "google/protobuf/wrappers.proto"
    optional string start_datetime = 3; // Option<DateTime<Utc>> -> parse in wooriDB
    optional string end_datetime = 4; // Option<DateTime<Utc>> -> parse in wooriDB
}
```


## Responses

### Error

```rust
Error {
  error_type: String, // Could use enumarations here like TxType
  error_message: String,
}
```

```rust
message Error {
  string error_type = 0;
  string error_message = 1;
}
```

### TX

**Enum TxType**
```rust
Create,
Insert,
UpdateSet,
UpdateContent,
Delete,
EvictEntity,
EvictEntityTree,
```

```rust
enum TxType {
    TX_TYPE_CREATE = 0;
    TX_TYPE_INSERT = 1;
    TX_TYPE_UPDATE_SET = 2;
    TX_TYPE_UPDATE_CONTENT = 3;
    TX_TYPE_DELETE = 4;
    TX_TYPE_EVICT_ENTITY = 5;
    TX_TYPE_EVICT_ENTITY_TREE = 6;
}
```


**TxResponse**
```rust
TxResponse {
  tx_type: TxType,
  entity: String,
  uuid: Option<Uuid>,
  state: String,
  message: String,
}
```

```rust
message TxResponse {
  TxType tx_type = 1;
  string entity = 2;
  optional string uuid = 3;
  string state = 4;
  string message = 5;
}
```

### Entity History
`BTreeMap<chrono::DateTime<Utc>, HashMap<std::string::String, Types>`

```rust
message SingleEntityHistory {
  string datetime = 1;
  repeat KeyPair map = 2;
}

message EntityHistory {
  repeat SingleEntityHistory entities = 1; 
}
```

### Query

```rust
Id(HashMap<String, Types>), // [x]
Intersect(HashMap<String, Types>), // [x]
Difference(HashMap<String, Types>), // [x]
Union(HashMap<String, Types>), // [x]
All(BTreeMap<Uuid, HashMap<String, Types>>), // [x]
Order(Vec<(Uuid, HashMap<String, Types>)>), // [x]
GroupBy(HashMap<String, BTreeMap<Uuid, HashMap<String, Types>>>), // [x]
OrderedGroupBy(HashMap<String, Vec<(Uuid, HashMap<String, Types>)>>), // [ ]
OptionOrder(Vec<(Uuid, Option<HashMap<String, Types>>)>), // [ ]
OptionGroupBy(HashMap<String, BTreeMap<Uuid, Option<HashMap<String, Types>>>>), // [ ]
OptionSelect(BTreeMap<Uuid, Option<HashMap<String, Types>>>), // [ ]
CheckValues(HashMap<String, bool>), // [ ]
TimeRange(BTreeMap<DateTime<Utc>, HashMap<String, Types>>), // [ ]
WithCount(CountResponse), // [ ]
DateSelect(HashMap<String, HashMap<String, Types>>), // [ ]
Join(Vec<HashMap<String, Types>>), // [ ]
```

* Count versions as well type `uint64`

message Order {
    message Attributes {
        map<string, string> values = 1;
    }
    repeated Attributes attributes = 1;
}
* Map query types to a message like Order, this is the only way to repeat map types.
* The value_type can be any type except another map.

```rust
enum QueryType {
    QUERY_TYPE_ID = 1;
    QUERY_TYPE_INTERSECT = 2;
    QUERY_TYPE_DIFFERENCE = 3;
    QUERY_TYPE_Union = 4;
    QUERY_TYPE_All = 5;
    QUERY_TYPE_Order = 6;
    QUERY_TYPE_GroupBy = 7;
    QUERY_TYPE_OrderedGroupBy = 8;
    QUERY_TYPE_OptionOrder = 9;
    QUERY_TYPE_OptionGroupBy = 10;
    QUERY_TYPE_OptionSelect = 11;
    QUERY_TYPE_CheckValues = 12;
    QUERY_TYPE_TimeRange = 13;
    QUERY_TYPE_WithCount = 14;
    QUERY_TYPE_DateSelect = 15;
    QUERY_TYPE_Join = 16;
}

message QueryPair {
  string key = 1;
  repeat KeyPair unique = 1;
}

message QueryGroup {
  string group_key = 1;
  repeat QueryPair multiple = 2;
}

message QueryResponseTypes {
  oneof types {
    repeat KeyPair unique = 1;
    repeat QueryPair multiple = 2;
    repeat QueryGroup group = 3;
  }
}

message QueryResponse {
  QueryType query_type = 1;
  QueryResponseTypes response = 2;
}
```

## Auth

CreateUserWithAdmin {
    pub admin_id: String,
    pub admin_password: String,
    pub user_info: UserInfo,
}

DeleteUsersWithAdmin {
    pub admin_id: String,
    pub admin_password: String,
    pub users_ids: Vec<Uuid>,
}

UserInfo {
    pub user_password: String,
    pub role: Vec<Role>,
}

pub enum Role {
    // Admin,
    User,
    Read,
    Write,
    History,
}

UserId {
    pub user_id: Uuid,
}

User {
    pub id: Uuid,
    pub user_password: String,
}
