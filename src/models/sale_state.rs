#[derive(DbEnum, Debug, Clone, PartialEq, Serialize, Deserialize, juniper::GraphQLEnum)]
pub enum SaleState {
    Draft,
    Approved,
    PartiallyPayed,
    Payed,
    Cancelled,
}

#[derive(Debug)]
pub enum Event {
    Approve,
    Cancel,
    PartiallyPay,
    Pay,
}

impl SaleState {
    pub fn next(self, event: Event) -> Result<SaleState, String> {
        match (self, event) {
            (SaleState::Draft, Event::Approve) => Ok(SaleState::Approved),
            (SaleState::Approved, Event::Pay) => Ok(SaleState::Payed),
            (SaleState::Approved, Event::PartiallyPay) => Ok(SaleState::PartiallyPayed),
            (SaleState::Approved, Event::Cancel) => Ok(SaleState::Cancelled),
            (SaleState::Payed, Event::Cancel) => Ok(SaleState::Cancelled),
            (SaleState::PartiallyPayed, Event::Cancel) => Ok(SaleState::Cancelled),
            (SaleState::PartiallyPayed, Event::Pay) => Ok(SaleState::Payed),
            (sale_state, sale_event) => Err(format!(
                "You can't {:#?} from {:#?} state",
                sale_event, sale_state
            )),
        }
    }
}
