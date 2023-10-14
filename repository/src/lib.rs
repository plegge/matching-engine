use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;
use protos::Order;

// Repository
pub trait OrderRepository: Send + Sync{
    fn place_order(&self, order: Order) -> Result<Order, &'static str>;
    fn cancel_order(&self, order_id: String) -> Result<(), &'static str>;
    fn get_order_status(&self, order_id: String) -> Result<Order, &'static str>;
    fn get_all_orders(&self, user_id: String) -> Result<Vec<Order>, &'static str>;
}

impl Debug for dyn OrderRepository {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OrderRepository{{{}}}", "_")
    }
}

#[derive(Debug, Default)]
pub struct InMemoryOrderRepository {
    orders: Mutex<HashMap<String, Order>>,
}

// Define a new method for InMemoryOrderRepository if it doesn't exist
impl InMemoryOrderRepository {
    pub fn new() -> Self {
        Self {
            orders: Mutex::new(HashMap::new())
        }
    }
}

impl OrderRepository for InMemoryOrderRepository {
    fn place_order(&self, order: Order) -> Result<Order, &'static str> {
        let mut orders = self.orders.lock().unwrap();
        orders.insert(order.id.clone(), order.clone());
        Ok(order)
    }

    fn cancel_order(&self, order_id: String) -> Result<(), &'static str> {
        let mut orders = self.orders.lock().unwrap();
        orders.remove(&order_id);
        Ok(())
    }

    fn get_order_status(&self, order_id: String) -> Result<Order, &'static str> {
        let orders = self.orders.lock().unwrap();
        orders.get(&order_id).cloned().ok_or("order not found")
    }

    fn get_all_orders(&self, user_id: String) -> Result<Vec<Order>, &'static str> {
        let orders = self.orders.lock().unwrap();
        Ok(orders.values()
            .filter(|&order| order.user_id == user_id)
            .cloned()
            .collect())
    }
}
