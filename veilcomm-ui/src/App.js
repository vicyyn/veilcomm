import React, { useState, useEffect } from 'react';
import { fetchRelays, fetchUsers, startRelay, startUser, sendCreate, sendExtend } from './requests';
import { RelayCard, UserCard, NewRelayPopup, NewUserPopup } from './components';
import styled from 'styled-components';
import { motion, AnimatePresence } from 'framer-motion';
import Draggable from 'react-draggable';
import './App.css';

const Dashboard = styled(motion.div)`
  width: 100%;
  min-height: 100vh;
  background-image: linear-gradient(#e8e8e8 1px, transparent 1px),
                    linear-gradient(90deg, #e8e8e8 1px, transparent 1px);
  background-size: 20px 20px;
  background-color: #f8f8f8;
  position: relative;
`;

const ControlPanel = styled.div`
  position: fixed;
  top: 0;
  right: 0;
  height: 100vh;
  padding: 20px;
  overflow-y: auto;
  z-index: 900;
`;

const AppContainer = styled.div`
`;

function App() {
  const [users, setUsers] = useState([]);
  const [relays, setRelays] = useState([]);
  const [userNickname, setUserNickname] = useState('');
  const [relayNickname, setRelayNickname] = useState('');
  const [selectedUser, setSelectedUser] = useState(null);
  const [selectedRelay, setSelectedRelay] = useState(null);
  const [isNewUserPopupOpen, setIsNewUserPopupOpen] = useState(false);
  const [isNewRelayPopupOpen, setIsNewRelayPopupOpen] = useState(false);
  const [update, setUpdate] = useState(false);

  useEffect(() => {
    async function fetchData() {
      let users = await fetchUsers();
      console.log('users:', users);
      setUsers(users);
      let relays = await fetchRelays();
      setRelays(relays);
    }
    fetchData();
  }, [update]);

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
    <AppContainer>
      <Dashboard>
        <AnimatePresence>
          {users.map(user => (
            <Draggable key={user.id} bounds="parent" defaultPosition={user.position}>
              <div style={{ position: 'absolute' }}>
                <UserCard
                  user={user}
                  isSelected={selectedUser && selectedUser.id === user.id}
                  onClick={() => setSelectedUser(user)}
                />
              </div>
            </Draggable>
          ))}
          {relays.map(relay => (
            <Draggable key={relay.id} bounds="parent" defaultPosition={relay.position}>
              <div style={{ position: 'absolute' }}>
                <RelayCard
                  relay={relay}
                  isSelected={selectedRelay && selectedRelay.id === relay.id}
                  onClick={() => setSelectedRelay(relay)}
                />
              </div>
            </Draggable>
          ))}
        </AnimatePresence>
      </Dashboard>

      <NewUserPopup
        isOpen={isNewUserPopupOpen}
        onClose={() => setIsNewUserPopupOpen(false)}
        onSubmit={startUser}
        setUpdate={setUpdate}
      />
      <NewRelayPopup
        isOpen={isNewRelayPopupOpen}
        onClose={() => setIsNewRelayPopupOpen(false)}
        onSubmit={startRelay}
        setUpdate={setUpdate}
      />

      <ControlPanel>
        <div><button style={{ fontSize: 24, minWidth: 200 }} onClick={() => setIsNewUserPopupOpen(true)}>New User</button></div>
        <div><button style={{ fontSize: 24, minWidth: 200 }} onClick={() => setIsNewRelayPopupOpen(true)}>New Relay</button></div>
      </ControlPanel>
    </AppContainer>
  );
}

export default App;