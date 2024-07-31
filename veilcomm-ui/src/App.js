import React, { useState, useEffect, useCallback } from 'react';
import { fetchRelays, fetchUsers, startRelay, startUser, sendCreate, sendExtend, fetchUserRelays, getUserLogs, getRelayLogs, sendEstablishRendezvous, sendEstablishIntroduction, sendBegin, sendIntroduce1 } from './requests';
import { RelayCard, UserCard, NewRelayPopup, NewUserPopup, DataPopup } from './components';
import styled from 'styled-components';
import { motion, AnimatePresence } from 'framer-motion';
import Draggable from 'react-draggable';
import { ToastContainer, toast } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';
import './App.css';
import { FiRefreshCw, FiInfo, FiGithub } from 'react-icons/fi';
import { generateRandomString } from './utils';

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
  width: 300px;
  position: fixed;
  top: 0;
  right: 0;
  height: 100vh;
  padding: 20px;
  overflow-y: auto;
  z-index: 40;
`;

const Section = styled.div`
  background-color: white;
  padding: 15px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  margin-bottom: 20px;
`;

const SectionTitle = styled.h3`
  margin-top: 0;
  margin-bottom: 10px;
  font-size: 18px;
`;

const ControlPanelContent = styled.div`
  margin-top: 10px;
`;

const Button = styled.button`
  font-size: 18px;
  margin: 10px 0;
  padding: 10px;
  width: 100%;
`;

const Select = styled.select`
  font-size: 16px;
  margin: 10px 0;
  padding: 5px;
  width: 100%;
`;

const ButtonGroup = styled.div`
  display: flex;
  justify-content: space-between;
  margin-bottom: 20px;
`;

const TopButton = styled.button`
  font-size: 26px;
  background-color: #FFA500;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  transition: background-color 0.3s, transform 0.3s;

  &:hover {
    background-color: #FF8C00;
    transform: scale(1.1);
  }
`;

const AppContainer = styled.div`
`;

function App() {
  const [users, setUsers] = useState([]);
  const [relays, setRelays] = useState([]);
  const [selectedUser, setSelectedUser] = useState(null);
  const [selectedRelay, setSelectedRelay] = useState(null);

  const [selectedCircuit, setSelectedCircuit] = useState(null);
  const [selectedSendUser, setSelectedSendUser] = useState(null);
  const [selectedReceiveRelay, setSelectedReceiveRelay] = useState(null);
  const [selectedExtendToRelay, setSelectedExtendToRelay] = useState(null);
  const [selectedBeginRelay, setSelectedBeginRelay] = useState(null);
  const [selectedRendezvousRelay, setSelectedRendezvousRelay] = useState(null);
  const [selectedCookie, setSelectedCookie] = useState(null);
  const [selectedIntroduction, setSelectedIntroduction] = useState(null);
  const [selectedStream, setSelectedStream] = useState(null);
  const [forUser, setForUser] = useState(null);

  const [circuits, setCircuits] = useState([]);
  const [cookies, setCookies] = useState([]);
  const [introductions, setIntroductions] = useState([]);
  const [streams, setStreams] = useState([]);

  const [isNewUserPopupOpen, setIsNewUserPopupOpen] = useState(false);
  const [isNewRelayPopupOpen, setIsNewRelayPopupOpen] = useState(false);
  const [selectedData, setSelectedData] = useState(null);
  const [popupPosition, setPopupPosition] = useState({ x: 0, y: 0 });
  const [update, setUpdate] = useState("");
  const [relaysLogs, setRelaysLogs] = useState([]);
  const [usersLogs, setUsersLogs] = useState([]);

  useEffect(() => {
    async function fetchData() {
      let users = await fetchUsers();
      setUsers(users);
      console.log(users)
      let relays = await fetchRelays();
      setRelays(relays);
      fetchUserRelays(users)
      let userLogs = await getUserLogs();
      setUsersLogs(userLogs);
      let relayLogs = await getRelayLogs();
      setRelaysLogs(relayLogs);
    }
    fetchData();
  }, [update]);

  const handleCardClick = useCallback((data, event) => {
    event.stopPropagation(); // Prevent the click from immediately closing the popup
    setSelectedData(data);
    setPopupPosition({ x: event.clientX, y: event.clientY });
  }, []);

  const getLogsForSelectedData = () => {
    if (selectedData.hasOwnProperty('id')) {
      return usersLogs.filter(log => log.nickname === selectedData.nickname)[0].logs;
    } else {
      return relaysLogs.filter(log => log.nickname === selectedData.nickname)[0].logs;
    }
  }

  const handleClosePopup = useCallback(() => {
    setSelectedData(null);
  }, []);

  useEffect(() => {
    const handleClickOutside = (event) => {
      if (selectedData) {
        handleClosePopup();
      }
    };

    document.addEventListener('click', handleClickOutside);
    return () => {
      document.removeEventListener('click', handleClickOutside);
    };
  }, [selectedData, handleClosePopup]);

  const handleSendCreate = () => {
    if (!selectedSendUser || !selectedReceiveRelay) {
      toast.error('Please select a user and a relay');
      return;
    }
    sendCreate(selectedSendUser, selectedReceiveRelay).then((circuit_id) => {
      setCircuits([...circuits, circuit_id]);
      setUpdate(generateRandomString());
    });
  };

  const handleSendExtend = () => {
    if (!selectedSendUser || !selectedReceiveRelay || !selectedExtendToRelay || !selectedCircuit) {
      toast.error('Please select a user and both relays for extend. Also select a circuit');
      return;
    }
    sendExtend(selectedSendUser, selectedReceiveRelay, selectedExtendToRelay, selectedCircuit);
    setUpdate(generateRandomString());
  };

  const handleSendEstablishRendezvous = () => {
    if (!selectedSendUser || !selectedReceiveRelay || !selectedCircuit) {
      toast.error('Please select a user and a relay');
      return;
    }
    sendEstablishRendezvous(selectedSendUser, selectedReceiveRelay, selectedCircuit).then((cookie) => {
      setUpdate(generateRandomString());
      setCookies([...cookies, cookie]);
    });
  }

  const handleSendEstablishIntroduction = () => {
    if (!selectedSendUser || !selectedReceiveRelay || !selectedCircuit) {
      toast.error('Please select a user and a relay');
      return;
    }
    sendEstablishIntroduction(selectedSendUser, selectedReceiveRelay, selectedCircuit).then((introduction_id) => {
      setIntroductions([...introductions, introduction_id]);
      setUpdate(generateRandomString);
    });
  }

  const handleSendBegin = () => {
    if (!selectedSendUser || !selectedReceiveRelay || !selectedCircuit || !selectedBeginRelay) {
      toast.error('Please select a user, relay, circuit and cookie');
      return;
    }
    sendBegin(selectedSendUser, selectedReceiveRelay, selectedCircuit, selectedBeginRelay).then((stream_id) => {
      setStreams([...streams, stream_id]);
      setUpdate(generateRandomString());
    });
  }

  const handleSendIntroduce1 = () => {
    if (!selectedSendUser || !selectedRelay || !selectedCircuit || !selectedRendezvousRelay || !selectedCookie || !selectedIntroduction || !selectedStream) {
      toast.error('Please select a user, relay, circuit, rendezvous relay, and provide a rendezvous cookie');
      return;
    }

    sendIntroduce1(
      selectedSendUser,
      selectedRelay,
      selectedIntroduction,
      selectedStream,
      selectedRendezvousRelay,
      selectedCookie,
      forUser.rsa_public,
      selectedCircuit
    ).then(() => {
      setUpdate(generateRandomString());
    });
  };

  return (
    <AppContainer>
      <ToastContainer autoClose={3000} />
      <Dashboard>
        <AnimatePresence>
          {users.map(user => (
            <Draggable key={user.id} bounds="parent" defaultPosition={user.position}>
              <div style={{ position: 'absolute' }}>
                <UserCard
                  user={user}
                  isSelected={selectedUser && selectedUser.id === user.id}
                  onClick={(event) => {
                    setSelectedUser(user);
                    handleCardClick(user, event);
                  }}
                />
              </div>
            </Draggable>
          ))}
          {relays.map(relay => (
            <Draggable key={relay.nickname} bounds="parent" defaultPosition={relay.position}>
              <div style={{ position: 'absolute' }}>
                <RelayCard
                  relay={relay}
                  isSelected={selectedRelay && selectedRelay.id === relay.id}
                  onClick={(event) => {
                    setSelectedRelay(relay);
                    handleCardClick(relay, event);
                  }}
                />
              </div>
            </Draggable>
          ))}
        </AnimatePresence>
      </Dashboard>

      {selectedData && (
        <DataPopup
          data={selectedData}
          position={popupPosition}
          getLogs={getLogsForSelectedData}
        />
      )}

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
        <ControlPanelContent>
          <ButtonGroup>
            <TopButton onClick={() => setUpdate(generateRandomString())} title="Restart">
              <FiRefreshCw />
            </TopButton>
            <TopButton onClick={() => { }} title="Info">
              <FiInfo />
            </TopButton>
            <TopButton onClick={() => { }} title="GitHub">
              <FiGithub />
            </TopButton>
          </ButtonGroup>
          <Section>
            <SectionTitle>New Entities</SectionTitle>
            <Button onClick={() => setIsNewUserPopupOpen(true)}>New User</Button>
            <Button onClick={() => setIsNewRelayPopupOpen(true)}>New Relay</Button>
          </Section>

          <Section>
            <SectionTitle>Common Selection</SectionTitle>
            <Select
              value={selectedSendUser ? selectedSendUser.id : ''}
              onChange={(e) => setSelectedSendUser(users.find(u => u.id === e.target.value))}
            >
              <option value="">Select User to Send</option>
              {users.map(user => (
                <option key={user.id} value={user.id}>{user.nickname}</option>
              ))}
            </Select>
            <Select
              value={selectedReceiveRelay ? selectedReceiveRelay.nickname : ''}
              onChange={(e) => setSelectedReceiveRelay(relays.find(r => r.nickname === e.target.value))}
            >
              <option value="">Select Relay to Receive</option>
              {relays.map(relay => (
                <option key={relay.nickname} value={relay.nickname}>{relay.nickname}</option>
              ))}
            </Select>
            <Select
              value={selectedCircuit ? selectedCircuit : ''}
              onChange={(e) => setSelectedCircuit(e.target.value)}
            >
              <option value="">Select Circuit</option>
              {circuits.map(circuit => (
                <option key={circuit} value={circuit}>{circuit}</option>
              ))}
            </Select>
            <Select
              value={selectedCookie ? selectedCookie : ''}
              onChange={(e) => setSelectedCookie(e.target.value)}
            >
              <option value="">Select Rendezvous Cookie</option>
              {cookies.map(cookie => (
                <option key={cookie} value={cookie}>{cookie}</option>
              ))}
            </Select>
            <Select
              value={selectedIntroduction ? selectedIntroduction : ''}
              onChange={(e) => setSelectedIntroduction(e.target.value)}
            >
              <option value="">Select Introduction Id</option>
              {introductions.map(introduction => (
                <option key={introduction} value={introduction}>{introduction}</option>
              ))}
            </Select>
            <Select
              value={selectedStream ? selectedStream : ''}
              onChange={(e) => setSelectedStream(e.target.value)}
            >
              <option value="">Select Stream</option>
              {streams.map(stream => (
                <option key={stream} value={stream}>{stream}</option>
              ))}
            </Select>
          </Section>

          <Section>
            <SectionTitle>Send Create</SectionTitle>
            <Button onClick={handleSendCreate}>Send Create</Button>
          </Section>

          <Section>
            <SectionTitle>Send Extend</SectionTitle>
            <Select
              value={selectedExtendToRelay ? selectedExtendToRelay.nickname : ''}
              onChange={(e) => setSelectedExtendToRelay(relays.find(r => r.nickname === e.target.value))}
            >
              <option value="">Select Relay to Extend To</option>
              {relays.map(relay => (
                <option key={relay.nickname} value={relay.nickname}>{relay.nickname}</option>
              ))}
            </Select>
            <Button onClick={handleSendExtend}>Send Extend</Button>
          </Section>

          <Section>
            <SectionTitle>Establish Rendezvous Point</SectionTitle>
            <Button onClick={handleSendEstablishRendezvous}>Establish Rendezvous</Button>
          </Section>

          <Section>
            <SectionTitle>Establish Introduction Point</SectionTitle>
            <Button onClick={handleSendEstablishIntroduction}>Establish Introduction</Button>
          </Section>

          <Section>
            <SectionTitle>Send Begin</SectionTitle>
            <Select
              value={selectedBeginRelay ? selectedBeginRelay.nickname : ''}
              onChange={(e) => setSelectedBeginRelay(relays.find(r => r.nickname === e.target.value))}
            >
              <option value="">Select Relay to Begin To</option>
              {relays.map(relay => (
                <option key={relay.nickname} value={relay.nickname}>{relay.nickname}</option>
              ))}
            </Select>
            <Button onClick={handleSendBegin}>Send Begin</Button>
          </Section>

          <Section>
            <SectionTitle>Send Introduce 1</SectionTitle>
            <Select
              value={forUser ? forUser.nickname : ''}
              onChange={(e) => setForUser(users.find(r => r.nickname === e.target.value))}
            >
              <option value="">Select User to Communicate with</option>
              {users.map(user => (
                <option key={user.nickname} value={user.nickname}>{user.nickname}</option>
              ))}
            </Select>
            <Select
              value={selectedRendezvousRelay ? selectedRendezvousRelay.nickname : ''}
              onChange={(e) => setSelectedRendezvousRelay(relays.find(r => r.nickname === e.target.value))}
            >
              <option value="">Select Rendezvous Relay</option>
              {relays.map(relay => (
                <option key={relay.nickname} value={relay.nickname}>{relay.nickname}</option>
              ))}
            </Select>
            <Button onClick={handleSendIntroduce1}>Send Introduce 1</Button>
          </Section>
        </ControlPanelContent>
      </ControlPanel>
    </AppContainer>
  );
}

export default App;