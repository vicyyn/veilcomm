import React, { useState, useEffect } from 'react';
import styled from 'styled-components';
import { motion, AnimatePresence } from 'framer-motion';
import UserCard from './UserCard';
import RelayCard from './RelayCard';
import './App.css';

const Dashboard = styled(motion.div)`
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 20px;
  padding: 20px;
`;

const ControlPanel = styled.div`
  display: flex;
  justify-content: space-between;
  padding: 20px;
  background-color: #f0f0f0;
`;

function App() {
  const [users, setUsers] = useState([]);
  const [relays, setRelays] = useState([]);
  const [selectedUser, setSelectedUser] = useState(null);
  const [selectedRelay, setSelectedRelay] = useState(null);
  const [selectedExtendRelay, setSelectedExtendRelay] = useState(null);
  const [newUserNickname, setNewUserNickname] = useState('');
  const [newRelayNickname, setNewRelayNickname] = useState('');
  const [newRelayAddress, setNewRelayAddress] = useState('');

  useEffect(() => {
    fetchUsers();
    fetchRelays();
  }, []);

  useEffect(() => {
    fetchUsers();
    fetchRelays();
  }, []);

  const fetchUsers = async () => {
    try {
      const response = await fetch('http://127.0.0.1:8081/users');
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      console.log(data)
      setUsers(data);
    } catch (error) {
      console.error('Error fetching users:', error);
    }
  };

  const fetchRelays = async () => {
    try {
      const response = await fetch('http://127.0.0.1:8081/relays');
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      setRelays(data);
    } catch (error) {
      console.error('Error fetching relays:', error);
    }
  };

  const fetchUserRelays = async (userId) => {
    if (!userId) {
      alert('Please select a user first');
      return;
    }
    try {
      const response = await fetch(`http://127.0.0.1:8081/users/${userId}/fetch_relays`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      alert('User relays fetched successfully');
    } catch (error) {
      console.error('Error fetching user relays:', error);
      alert('Failed to fetch user relays');
    }
  };

  const startUser = async () => {
    if (!newUserNickname) {
      alert('Please enter a nickname for the new user');
      return;
    }
    try {
      const response = await fetch('http://127.0.0.1:8081/start_user', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ nickname: newUserNickname }),
      });
      if (response.ok) {
        alert('User started successfully');
        setNewUserNickname('');
        fetchUsers();
      } else {
        alert('Failed to start user');
      }
    } catch (error) {
      console.error('Error:', error);
      alert('An error occurred while starting the user');
    }
  };

  const startRelay = async () => {
    if (!newRelayNickname || !newRelayAddress) {
      alert('Please enter both nickname and address for the new relay');
      return;
    }
    try {
      const response = await fetch('http://127.0.0.1:8081/start_relay', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          nickname: newRelayNickname,
          address: newRelayAddress
        }),
      });
      if (response.ok) {
        alert('Relay started successfully');
        setNewRelayNickname('');
        setNewRelayAddress('');
        fetchRelays();
      } else {
        alert('Failed to start relay');
      }
    } catch (error) {
      console.error('Error:', error);
      alert('An error occurred while starting the relay');
    }
  };

  const sendCreate = async () => {
    if (!selectedUser || !selectedRelay) {
      alert('Please select a user and a relay');
      return;
    }
    try {
      const response = await fetch(`http://127.0.0.1:8081/users/${selectedUser.id}/send_create_to_relay/`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          relay_socket: selectedRelay.address
        }),
      });
      if (response.ok) {
        alert('Create sent successfully');
      } else {
        alert('Failed to send create');
      }
    } catch (error) {
      console.error('Error:', error);
      alert('An error occurred while sending create');
    }
  };

  const sendExtend = async () => {
    if (!selectedUser || !selectedRelay || !selectedExtendRelay) {
      alert('Please select a user, a relay to send to, and a relay to extend to');
      return;
    }
    try {
      const response = await fetch(`http://127.0.0.1:8081/users/${selectedUser.id}/send_extend_to_relay/`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          relay_socket: selectedRelay.address,
          extend_to: selectedExtendRelay.address
        }),
      });
      if (response.ok) {
        alert('Extend sent successfully');
      } else {
        alert('Failed to send extend');
      }
    } catch (error) {
      console.error('Error:', error);
      alert('An error occurred while sending extend');
    }
  };


  const handleSend = async (operation) => {
    if (!selectedUser) {
      alert('Please select a user first');
      return;
    }

    try {
      const response = await fetch(`http://localhost:8081/users/${selectedUser.id}/${operation}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({}), // Add necessary payload
      });
      if (response.ok) {
        alert(`${operation} successful`);
      } else {
        alert(`${operation} failed`);
      }
    } catch (error) {
      console.error('Error:', error);
      alert('An error occurred');
    }
  };

  return (
    <div className="App">
      <h1>VeilComm Dashboard</h1>
      <Dashboard>
        <AnimatePresence>
          {users.map(user => (
            <UserCard
              key={user.id}
              user={user}
              isSelected={selectedUser && selectedUser.id === user.id}
              onClick={() => setSelectedUser(user)}
            />
          ))}
          {relays.map(relay => (
            <RelayCard
              key={relay.id}
              relay={relay}
              isSelected={selectedRelay && selectedRelay.id === relay.id}
              onClick={() => setSelectedRelay(relay)}
            />
          ))}
        </AnimatePresence>
      </Dashboard>

      <div className="start-section">
        <h2>Start New User</h2>
        <input
          type="text"
          value={newUserNickname}
          onChange={(e) => setNewUserNickname(e.target.value)}
          placeholder="Enter user nickname"
        />
        <button onClick={startUser}>Start User</button>

        <h2>Start New Relay</h2>
        <input
          type="text"
          value={newRelayNickname}
          onChange={(e) => setNewRelayNickname(e.target.value)}
          placeholder="Enter relay nickname"
        />
        <input
          type="text"
          value={newRelayAddress}
          onChange={(e) => setNewRelayAddress(e.target.value)}
          placeholder="Enter relay address"
        />
        <button onClick={startRelay}>Start Relay</button>
      </div>

      <div className="users-section">
        <h2>Users</h2>
        <ul>
          {users.map(user => (
            <li
              key={user.id}
              onClick={() => setSelectedUser(user)}
              className={selectedUser && selectedUser.id === user.id ? 'selected' : ''}
            >
              {user.nickname}
            </li>
          ))}
        </ul>
      </div>

      <div className="relays-section">
        <h2>Relays</h2>
        <ul>
          {relays.map(relay => (
            <li key={relay.id}>{relay.nickname} - {relay.address}</li>
          ))}
        </ul>
      </div>

      <div className="operations-section">
        <h2>Operations</h2>
        <div>
          <h3>Send Create</h3>
          <select onChange={(e) => setSelectedRelay(relays.find(r => r.nickname === e.target.value))}>
            <option value="">Select a relay</option>
            {relays.map(relay => (
              <option key={relay.id} value={relay.id}>{relay.nickname}</option>
            ))}
          </select>
          <button onClick={sendCreate}>Send Create</button>
        </div>
        <div>
          <h3>Send Extend</h3>
          <select onChange={(e) => setSelectedRelay(relays.find(r => r.nickname === e.target.value))}>
            <option value="">Select a relay to send to</option>
            {relays.map(relay => (
              <option key={relay.id} value={relay.id}>{relay.nickname}</option>
            ))}
          </select>
          <select onChange={(e) => setSelectedExtendRelay(relays.find(r => r.nickname === e.target.value))}>
            <option value="">Select a relay to extend to</option>
            {relays.map(relay => (
              <option key={relay.id} value={relay.id}>{relay.nickname}</option>
            ))}
          </select>
          <button onClick={sendExtend}>Send Extend</button>
        </div>
        <button onClick={() => fetchUserRelays(selectedUser?.id)}>Fetch User Relays</button>
        <button onClick={() => handleSend('fetch_relays')}>Fetch Relays</button>
        <button onClick={() => handleSend('establish_circuit')}>Establish Circuit</button>
        <button onClick={() => handleSend('send_establish_introduction_to_relay')}>Send Establish Introduction</button>
        <button onClick={() => handleSend('add_introduction_point')}>Add Introduction Point</button>
        <button onClick={() => handleSend('update_introduction_points')}>Update Introduction Points</button>
        <button onClick={() => handleSend('get_circuit_id_for_rendezvous')}>Get Circuit ID for Rendezvous</button>
        <button onClick={() => handleSend('send_rendezvous1_to_relay')}>Send Rendezvous1</button>
        <button onClick={() => handleSend('send_establish_rendezvous_to_relay')}>Send Establish Rendezvous</button>
        <button onClick={() => handleSend('send_begin_to_relay')}>Send Begin</button>
        <button onClick={() => handleSend('send_introduce1_to_relay')}>Send Introduce1</button>
        <button onClick={() => handleSend('send_data_to_relay')}>Send Data</button>
      </div >
    </div >
  );
}

export default App;