use bigneon_db::models::*;
use config::Config;
use diesel::pg::PgConnection;
use errors::*;
use utils::communication::CommAddress;
use utils::communication::Communication;
use utils::communication::CommunicationType;
use utils::deep_linker::DeepLinker;

pub fn transfer_cancelled(
    config: &Config,
    phone: String,
    from_user: &User,
    conn: &PgConnection,
) -> Result<(), BigNeonError> {
    let source = CommAddress::from(config.communication_default_source_phone.clone());
    let destinations = CommAddress::from(phone);
    let body = format!(
        "{} has cancelled their transferred tickets.",
        from_user.full_name()
    );
    Communication::new(
        CommunicationType::Sms,
        body,
        None,
        Some(source),
        destinations,
        None,
        None,
    )
    .queue(conn)
}

pub fn transfer_drip_reminder(
    phone: String,
    transfer: &Transfer,
    event: &Event,
    config: &Config,
    conn: &PgConnection,
    deep_linker: &DeepLinker,
) -> Result<(), BigNeonError> {
    let receive_tickets_link = format!(
        "{}/tickets/receive?sender_user_id={}&transfer_key={}&num_tickets={}&signature={}",
        config.front_end_url.clone(),
        transfer.source_user_id,
        transfer.transfer_key,
        transfer.transfer_tickets(conn)?.len(),
        transfer.signature(conn)?
    );

    let shortened_link = deep_linker.create_deep_link(&receive_tickets_link)?;
    let source = CommAddress::from(config.communication_default_source_phone.clone());
    let destinations = CommAddress::from(phone);
    let body = format!(
        "{} Follow this link to receive them: {}",
        transfer.drip_header(event, SourceOrDestination::Destination, false, conn)?,
        shortened_link
    );
    Communication::new(
        CommunicationType::Sms,
        body,
        None,
        Some(source),
        destinations,
        None,
        None,
    )
    .queue(conn)
}

pub fn send_tickets(
    config: &Config,
    phone: String,
    num_tickets: u32,
    transfer_key: &str,
    signature: &str,
    from_user: &User,
    conn: &PgConnection,
    deep_linker: &DeepLinker,
) -> Result<(), BigNeonError> {
    let receive_tickets_link = format!(
        "{}/tickets/transfers/receive?sender_user_id={}&transfer_key={}&num_tickets={}&signature={}",
        config.front_end_url.clone(),
        from_user.id,
        transfer_key,
        num_tickets,
        signature
    );

    let shortened_link = deep_linker.create_deep_link(&receive_tickets_link)?;

    let source = CommAddress::from(config.communication_default_source_phone.clone());
    let destinations = CommAddress::from(phone);
    let body = format!(
        "{} has sent you some tickets. Follow this link to receive them: {}",
        from_user.full_name(),
        shortened_link
    );
    Communication::new(
        CommunicationType::Sms,
        body,
        None,
        Some(source),
        destinations,
        None,
        None,
    )
    .queue(conn)
}
