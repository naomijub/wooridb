# Authorization and Authentication
Authentication and authorization only work with release mode, so `cargo run --release` is required. 

Some environment variables are also required:
```
AUTH_HASHING_COST=8 
ADMIN=your_admin
ADMIN_PASSWORD=your_password
```

* `AUTH_HASHING_COST` is the hashing cost for password in the authentication system.
* `ADMIN` is the admin's user id.
* `ADMIN_PASSWORD` is the admin's user password.

## Creating new users
* `ADMIN` is the only user role capable of creating new users. For now there can only be one `ADMIN`.

To create a new user, POST at `/auth/createUser` with your admin credentials and the new user info as follows (in RON format):
```ron
(
  admin_id: "your_admin",
  admin_password: "your_password",
  user_info: (
    user_password: "my_password",
    role: [User,],
  ),
)
```
User information consists of the user's password to be used and the user's roles. Remember to always put `,` at the end. 
Response to this request will be `(user_id: \"<some-uuid>\",)`, containing the user's unique ID.

### Available user roles are:
- `ADMIN` - works primarily at `/auth/createUser`.
- `USER` - works on all `/wql/query`, `/wql/tx` and `/auth/putUserSession`.
- `WRITE` - only works on `/wql/tx` and `/auth/putUserSession`.
- `READ` - only works on `/wql/query` and `/auth/putUserSession`.
- `HISTORY` - only works on `/entity-history`.
- New roles to be added as needed.

### Getting a session token
To make a request at WQL endpoints you need a session token that will expire within 3600 seconds. To retrieve a session token you need to `PUT` at endpoint `/auth/putUserSession` your user credentials as follows (in RON format):
```ron
(id: "<user_id>", user_password: "<user_password>",)
```
Response will be a plain/text with your token.

### Making auth requests to `/wql/tx` and `/wql/query`.

To avoid authentication and authorization errors, add your token to the authorization bearer header, `Authorization: Bearer <your session token>`. 
Your user needs the correct session token and the correct role for this request.

### TODOs:
* [ ] Adding other admins and removing admins is not yet implemented.
* [ ] Configure session token expiration time.