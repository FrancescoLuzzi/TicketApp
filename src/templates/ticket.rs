use askama::Template;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "ticket/ticket_page.html")]
pub struct TicketPage {
    pub tickets: Vec<Ticket>,
}

#[derive(Template)]
#[template(path = "ticket/ticket.html")]
pub struct Ticket {
    pub amount: f32,
    pub description: String,
    pub type_str: String,
    pub type_id: Uuid,
    pub id: Uuid,
}
