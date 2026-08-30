mod dbclient;
mod server;

pub mod proto {
    pub mod connections {
        include!(concat!(env!("OUT_DIR"), "/dbclient.connections.rs"));
    }
    pub mod objects {
        include!(concat!(env!("OUT_DIR"), "/dbclient.objects.rs"));
    }
    pub mod queries {
        include!(concat!(env!("OUT_DIR"), "/dbclient.queries.rs"));
    }
    pub mod common {
        include!(concat!(env!("OUT_DIR"), "/dbclient.common.rs"));
    }
}

// enum CommandError {
//
// }
//
// trait Command {
//     fn connections() -> impl Connections;
//     fn objects() -> impl Objects;
//     fn queries() -> impl Queries;
// }
//
// trait Connections {
//     fn get_available_list() -> Result<Vec<Connection>, CommandError>;
// }
//
// struct Connection {
//     id: String
// }
//
// trait Objects {
//     fn get_objects_of_connection(conn: &Connection) -> Result<Vec<DbObject>, CommandError>;
//     fn add_object_to_connection(conn: &Connection, obj: DbObject) -> Result<(), CommandError>;
// }
//
// struct DbObject {
//
// }
//
// trait Queries {
//     fn execute_query(q: &Query) -> Result<QueryResult, CommandError>;
// }
//
// struct Query {
//
// }
//
// struct QueryResult {
//
// }
