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
  string key = 0;
  WqpTypes value = 1;
}

message WqlTypes {
  oneof types {
    bytes char = 0;
    int64 int = 1;
    string string = 2;
    string uuid = 3;
    double float = 4;
    bool boolean = 5;
    // Vector(Vec<Types>) = 6; cyclic?
    repeated WqpTypes vec = 6;
    // Map(HashMap<String, Types>) = 7; cyclic?? or Map like in Order
    repeat KeyPair map = 7;
    string precise = 8;
    string datetime = 9; // or datetime
    Empty nil = 10;
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


### Query

Id(HashMap<String, Types>),
Intersect(HashMap<String, Types>),
Difference(HashMap<String, Types>),
Union(HashMap<String, Types>),
All(BTreeMap<Uuid, HashMap<String, Types>>),
Order(Vec<(Uuid, HashMap<String, Types>)>),
GroupBy(HashMap<String, BTreeMap<Uuid, HashMap<String, Types>>>),
OrderedGroupBy(HashMap<String, Vec<(Uuid, HashMap<String, Types>)>>),
OptionOrder(Vec<(Uuid, Option<HashMap<String, Types>>)>),
OptionGroupBy(HashMap<String, BTreeMap<Uuid, Option<HashMap<String, Types>>>>),
OptionSelect(BTreeMap<Uuid, Option<HashMap<String, Types>>>),
CheckValues(HashMap<String, bool>),
TimeRange(BTreeMap<DateTime<Utc>, HashMap<String, Types>>),
WithCount(CountResponse),
DateSelect(HashMap<String, HashMap<String, Types>>),
Join(Vec<HashMap<String, Types>>),

* Count versions as well type `uint64`

message Order {
    message Attributes {
        map<string, string> values = 1;
    }
    repeated Attributes attributes = 1;
}
* Map query types to a message like Order, this is the only way to repeat map types.
* The value_type can be any type except another map.


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
