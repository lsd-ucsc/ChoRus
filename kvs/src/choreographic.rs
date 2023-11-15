use crate::shared::*;

use chorus_lib::core::{ChoreoOp, Choreography, Located, LocationSet};

pub struct PrimaryBackupKvsChoreography<'a> {
    pub state: (Located<&'a State, Primary>, Located<&'a State, Backup>),
    pub request: Located<Request, Client>,
}
impl<'a> Choreography<Located<Response, Client>> for PrimaryBackupKvsChoreography<'a> {
    type L = L;
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<Response, Client> {
        let request = op.comm(Client, Primary, &self.request);
        struct DoBackup<'a> {
            request: Located<Request, Primary>,
            state: Located<&'a State, Backup>,
        }
        impl<'a> Choreography for DoBackup<'a> {
            type L = LocationSet!(Primary, Backup);
            fn run(self, op: &impl ChoreoOp<Self::L>) {
                let is_mutating = op.locally(Primary, |un| un.unwrap(&self.request).is_mutating());
                let is_mutating = op.broadcast(Primary, is_mutating);
                if is_mutating {
                    let request = op.comm(Primary, Backup, &self.request);
                    let response = op.locally(Backup, |un| {
                        handle_request(*un.unwrap(&self.state), un.unwrap(&request))
                    });
                    op.comm(Backup, Primary, &response);
                }
            }
        }
        op.enclave(DoBackup {
            request: request.clone(),
            state: self.state.1,
        });
        let response = op.locally(Primary, |un| {
            handle_request(*un.unwrap(&self.state.0), un.unwrap(&request))
        });
        op.comm(Primary, Client, &response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chorus_lib::core::Runner;

    #[test]
    fn test_kvs_choreography() {
        let runner = Runner::<L>::new();
        let primary_state = State::default();
        let backup_state = State::default();
        runner.run(PrimaryBackupKvsChoreography {
            state: (runner.local(&primary_state), runner.local(&backup_state)),
            request: runner.local(Request::Put("Hello".to_string(), "World".to_string())),
        });
        assert_eq!(primary_state.borrow().get("Hello").unwrap(), "World");
        assert_eq!(backup_state.borrow().get("Hello").unwrap(), "World");
        let response = runner.run(PrimaryBackupKvsChoreography {
            state: (runner.local(&primary_state), runner.local(&backup_state)),
            request: runner.local(Request::Get("Hello".to_string())),
        });
        let response = runner.unwrap(response);
        assert_eq!(response, Some("World".to_string()));
    }
}
