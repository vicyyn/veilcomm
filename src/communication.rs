use crate::RelayCell;
use anyhow::Result;
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{mpsc, Mutex},
};
use uuid::Uuid;

lazy_static! {
    pub static ref communication: Communication = Communication {
        connections: Mutex::new(HashMap::new()),
    };
}

pub struct Communication {
    connections: Mutex<HashMap<Uuid, mpsc::Sender<(Uuid, RelayCell)>>>,
}

impl Communication {
    pub fn register(id: Uuid) -> mpsc::Receiver<(Uuid, RelayCell)> {
        let (tx, rx) = mpsc::channel();
        communication.connections.lock().unwrap().insert(id, tx);
        rx
    }

    pub fn send(sender: Uuid, receiver: Uuid, cell: RelayCell) -> Result<()> {
        let connections = communication.connections.lock().unwrap();
        if let Some(tx) = connections.get(&receiver) {
            tx.send((sender, cell))?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Receiver not found"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    // Helper function to create a mock RelayCell
    fn create_mock_relay_cell() -> RelayCell {
        RelayCell {
            circuit_id: Uuid::new_v4(),
            payload: vec![1, 2, 3],
        }
    }

    #[test]
    fn test_register_new_connection() {
        let id = Uuid::new_v4();
        Communication::register(id);
        assert!(communication.connections.lock().unwrap().contains_key(&id));
    }

    #[test]
    fn test_register_multiple_connections() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let _rx1 = Communication::register(id1);
        let _rx2 = Communication::register(id2);
        let connections = communication.connections.lock().unwrap();
        assert!(connections.contains_key(&id1));
        assert!(connections.contains_key(&id2));
        assert_eq!(connections.len(), 2);
    }

    #[test]
    fn test_send_message() {
        let sender_id = Uuid::new_v4();
        let receiver_id = Uuid::new_v4();
        let rx = Communication::register(receiver_id);
        let cell = create_mock_relay_cell();

        Communication::send(sender_id, receiver_id, cell.clone()).unwrap();

        let (received_sender, received_cell) = rx.recv().unwrap();
        assert_eq!(received_sender, sender_id);
        assert_eq!(received_cell.circuit_id, cell.circuit_id);
        assert_eq!(received_cell.payload, cell.payload);
    }

    #[test]
    fn test_send_multiple_messages() {
        let sender_id = Uuid::new_v4();
        let receiver_id = Uuid::new_v4();
        let rx = Communication::register(receiver_id);
        let cell1 = create_mock_relay_cell();
        let cell2 = create_mock_relay_cell();

        Communication::send(sender_id, receiver_id, cell1.clone()).unwrap();
        Communication::send(sender_id, receiver_id, cell2.clone()).unwrap();

        let (received_sender1, received_cell1) = rx.recv().unwrap();
        let (received_sender2, received_cell2) = rx.recv().unwrap();

        assert_eq!(received_sender1, sender_id);
        assert_eq!(received_cell1.circuit_id, cell1.circuit_id);
        assert_eq!(received_cell1.payload, cell1.payload);

        assert_eq!(received_sender2, sender_id);
        assert_eq!(received_cell2.circuit_id, cell2.circuit_id);
        assert_eq!(received_cell2.payload, cell2.payload);
    }

    #[test]
    #[should_panic]
    fn test_send_to_nonexistent_receiver() {
        let sender_id = Uuid::new_v4();
        let nonexistent_receiver_id = Uuid::new_v4();
        let cell = create_mock_relay_cell();

        Communication::send(sender_id, nonexistent_receiver_id, cell).unwrap();
    }

    #[test]
    fn test_multiple_senders_single_receiver() {
        let receiver_id = Uuid::new_v4();
        let rx = Communication::register(receiver_id);
        let sender_ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();

        for sender_id in &sender_ids {
            let cell = create_mock_relay_cell();
            Communication::send(*sender_id, receiver_id, cell).unwrap();
        }

        for _ in &sender_ids {
            let (received_sender, _) = rx.recv().unwrap();
            assert!(sender_ids.contains(&received_sender));
        }
    }

    #[test]
    fn test_concurrent_registrations() {
        let mut handles = vec![];
        let num_threads = 10;

        for _ in 0..num_threads {
            let handle = thread::spawn(|| {
                let id = Uuid::new_v4();
                let _rx = Communication::register(id);
                id
            });
            handles.push(handle);
        }

        let ids: Vec<Uuid> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let connections = communication.connections.lock().unwrap();

        for id in ids {
            assert!(connections.contains_key(&id));
        }
        assert_eq!(connections.len(), num_threads);
    }

    #[test]
    fn test_concurrent_sends() {
        let receiver_id = Uuid::new_v4();
        let rx = Communication::register(receiver_id);
        let num_senders = 10;
        let mut handles = vec![];

        for _ in 0..num_senders {
            let handle = thread::spawn(move || {
                let sender_id = Uuid::new_v4();
                let cell = create_mock_relay_cell();
                Communication::send(sender_id, receiver_id, cell).unwrap();
                sender_id
            });
            handles.push(handle);
        }

        let sender_ids: Vec<Uuid> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        for _ in 0..num_senders {
            let (received_sender, _) = rx.recv().unwrap();
            assert!(sender_ids.contains(&received_sender));
        }
    }

    #[test]
    #[should_panic]
    fn test_channel_closed() {
        let receiver_id = Uuid::new_v4();
        let rx = Communication::register(receiver_id);
        drop(rx);

        let sender_id = Uuid::new_v4();
        let cell = create_mock_relay_cell();

        Communication::send(sender_id, receiver_id, cell).unwrap();
    }

    #[test]
    fn test_reregister_same_id() {
        let id = Uuid::new_v4();
        let _rx1 = Communication::register(id);
        let _rx2 = Communication::register(id);

        let connections = communication.connections.lock().unwrap();
        assert_eq!(connections.len(), 1);
        assert!(connections.contains_key(&id));
    }

    #[test]
    fn test_large_payload() {
        let sender_id = Uuid::new_v4();
        let receiver_id = Uuid::new_v4();
        let rx = Communication::register(receiver_id);

        let large_payload = vec![0u8; 1_000_000]; // 1 MB payload
        let cell = RelayCell {
            circuit_id: Uuid::new_v4(),
            payload: large_payload.clone(),
        };

        Communication::send(sender_id, receiver_id, cell).unwrap();

        let (received_sender, received_cell) = rx.recv().unwrap();
        assert_eq!(received_sender, sender_id);
        assert_eq!(received_cell.payload, large_payload);
    }

    #[test]
    fn test_send_after_delay() {
        let sender_id = Uuid::new_v4();
        let receiver_id = Uuid::new_v4();
        let rx = Communication::register(receiver_id);
        let cell = create_mock_relay_cell();
        let payload = cell.payload.clone();
        let circuit_id = cell.circuit_id;

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            Communication::send(sender_id, receiver_id, cell).unwrap();
        });

        let (received_sender, received_cell) = rx.recv().unwrap();
        assert_eq!(received_sender, sender_id);
        assert_eq!(received_cell.circuit_id, circuit_id);
        assert_eq!(received_cell.payload, payload);
    }
}
