use std::fmt::Debug;

use protos::Order;

// Producer
pub trait OrderProducer: Send + Sync{
    fn place_order(&self, order: Order) -> Result<Order, &'static str>;
    fn cancel_order(&self, order_id: String) -> Result<String, &'static str>;
}

impl Debug for dyn OrderProducer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OrderProducer{{{}}}", "_")
    }
}


#[derive(Debug, Default)]
pub struct KafkaProducer {

}

impl KafkaProducer {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl OrderProducer for KafkaProducer {
    fn place_order(&self, order: Order) -> Result<Order, &'static str> {
        // kafka produce Order message
        Ok(order)
    }

    fn cancel_order(&self, order_id: String) -> Result<String, &'static str> {
        // kafka produce Cancel message
        Ok(order_id)
    }

}
