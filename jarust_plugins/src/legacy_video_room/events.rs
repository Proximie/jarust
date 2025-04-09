use jarust_core::prelude::JaResponse;
use jarust_interface::japrotocol::GenericEvent;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PluginEvent {
    GenericEvent(GenericEvent),
}

impl TryFrom<JaResponse> for PluginEvent {
    type Error = jarust_interface::Error;

    fn try_from(_: JaResponse) -> Result<Self, Self::Error> {
        todo!("Implement conversion from JaResponse to PluginEvent");
    }
}
