import React, { useState, useEffect, useCallback } from 'react';
import { startRelay, startUser, sendCreate, sendExtend, sendEstablishRendezvous, sendEstablishIntroduction, sendBegin, sendIntroduce1, sendRendezvous1, sendData, getState } from './requests';
import { Card, ConnectionLines, DataPopup } from './components';
import styled from 'styled-components';
import { motion, AnimatePresence } from 'framer-motion';
import Draggable from 'react-draggable';
import { ToastContainer, toast } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';
import './App.css';
import { FiRefreshCw, FiInfo, FiGithub } from 'react-icons/fi';
import { generateRandomString } from './utils';
import { Position, RelayState, UserState } from './data';

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
  width: 200px;
  position: fixed;
  top: 0;
  right: 0;
  height: 100vh;
  padding: 10px;
  overflow-y: auto;
  z-index: 40;
  font-size: 10px;
`;

const Input = styled.input`
  font-size: 10px;
  margin: 5px 0;
  padding: 3px;
  width: 100%;
`;

const Section = styled.div`
  background-color: white;
  padding: 10px;
  border-radius: 6px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  margin-bottom: 15px;
`;

const SectionTitle = styled.h3`
  margin-top: 0;
  margin-bottom: 5px;
  font-size: 14px;
`;

const ControlPanelContent = styled.div`
  margin-top: 5px;
`;

const Button = styled.button`
  font-size: 12px;
  margin: 5px 0;
  padding: 5px;
  width: 100%;
`;

const Select = styled.select`
  font-size: 12px;
  margin: 5px 0;
  padding: 3px;
  width: 100%;
`;

const ButtonGroup = styled.div`
  display: flex;
  justify-content: space-between;
  margin-bottom: 15px;
`;

const TopButton = styled.button`
  font-size: 20px;
  background-color: #FFA500;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  transition: background-color 0.3s, transform 0.3s;
  padding: 10px;

  &:hover {
    background-color: #FF8C00;
    transform: scale(1.1);
  }
`;

const ConnectionLinesWrapper = styled.div`
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 1;
`;

const CardsWrapper = styled.div`
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 2;
`;

const CardContainer = styled.div`
  position: absolute;
  z-index: 3;
`;

const AppContainer = styled.div`
`;

function App() {
  const [users, setUsers] = useState<UserState[]>([]);
  const [relays, setRelays] = useState<RelayState[]>([]);
  const [positions, setPositions] = useState<{ [key: string]: Position }>({});

  const [selectedCreateRelay, setSelectedCreateRelay] = useState<RelayState | undefined>(undefined);
  const [selectedCircuit, setSelectedCircuit] = useState<string>("");
  const [selectedSendUser, setSelectedSendUser] = useState<UserState | undefined>(undefined);
  const [selectedReceiveRelay, setSelectedReceiveRelay] = useState<RelayState | undefined>(undefined);
  const [selectedExtendToRelay, setSelectedExtendToRelay] = useState<RelayState | undefined>(undefined);
  const [selectedBeginRelay, setSelectedBeginRelay] = useState<RelayState | undefined>(undefined);

  const [isRendezvousRelays, setIsRendezvousRelays] = useState<Record<string, boolean>>({});
  const [isIntroductionRelays, setIsIntroductionRelays] = useState<Record<string, boolean>>({});

  const [selectedCookie, setSelectedCookie] = useState<string>("");
  const [selectedIntroduction, setSelectedIntroduction] = useState<string>("");
  const [selectedStream, setSelectedStream] = useState<string>("");
  const [data, setData] = useState<string>("");

  const [forUser, setForUser] = useState<UserState | undefined>(undefined);
  const [selectedData, setSelectedData] = useState<RelayState | UserState | undefined>(undefined);
  const [popupPosition, setPopupPosition] = useState({ x: 0, y: 0 });
  const [update, setUpdate] = useState("");

  useEffect(() => {
    const savedPositions = localStorage.getItem('cardPositions');
    if (savedPositions) {
      setPositions(JSON.parse(savedPositions));
    }
  }, []);

  useEffect(() => {
    localStorage.setItem('cardPositions', JSON.stringify(positions));
  }, [positions]);

  useEffect(() => {
    async function updateState() {
      const newState = await getState();
      console.log(newState);

      setPositions(prevPositions => {
        const updatedPositions = { ...prevPositions };

        newState?.user_states.forEach(user => {
          if (!updatedPositions[user.id]) {
            updatedPositions[user.id] = {
              x: Math.floor(Math.random() * 500),
              y: Math.floor(Math.random() * 500)
            };
          }
        });

        newState?.relay_states.forEach(relay => {
          if (!updatedPositions[relay.id]) {
            updatedPositions[relay.id] = {
              x: Math.floor(Math.random() * 500),
              y: Math.floor(Math.random() * 500)
            };
          }
        });

        return updatedPositions;
      });

      setUsers(newState?.user_states || []);
      setRelays(newState?.relay_states || []);

      const rendezvousRelays: Record<string, boolean> = {};
      const introductionRelays: Record<string, boolean> = {};

      newState?.relay_states.forEach(relay => {
        if (relay.is_rendezvous_point) {
          rendezvousRelays[relay.id] = true;
        }
        if (relay.is_introduction_point) {
          introductionRelays[relay.id] = true;
        }
      });

      setIsRendezvousRelays(rendezvousRelays);
      setIsIntroductionRelays(introductionRelays);
    }

    updateState();
  }, [update]);

  useEffect(() => {
    if (selectedCircuit && selectedSendUser) {
      const user = users.find(u => u.id === selectedSendUser.id);
      if (user && user.circuits[selectedCircuit]) {
        const firstRelayId = user.circuits[selectedCircuit][0];
        const firstRelay = relays.find(r => r.id === firstRelayId);
        setSelectedReceiveRelay(firstRelay);
      }
    }
  }, [selectedCircuit, selectedSendUser, users, relays]);

  const handleDrag = (id: string, data: { x: number; y: number }) => {
    setPositions(prevPositions => ({
      ...prevPositions,
      [id]: { x: data.x, y: data.y }
    }));
  };

  const handleClosePopup = useCallback(() => {
    setSelectedData(undefined);
  }, []);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (selectedData) {
        handleClosePopup();
      }
    };

    document.addEventListener('click', handleClickOutside);
    return () => {
      document.removeEventListener('click', handleClickOutside);
    };
  }, [selectedData, handleClosePopup]);

  const handleStartUser = () => {
    let user_next_number = Number(users.length) + 1;
    startUser("User " + user_next_number).then(() => {
      setUpdate(generateRandomString());
    });
  }

  const handleStartRelay = () => {
    let relay_next_number = Number(relays.length) + 1;
    startRelay("Relay " + relay_next_number).then(() => {
      setUpdate(generateRandomString());
    });
  }


  const handleSendCreate = () => {
    if (!selectedSendUser || !selectedCreateRelay) {
      toast.error('Please select a user and a relay for create');
      return;
    }
    sendCreate(selectedSendUser, selectedCreateRelay).then(() => {
      setUpdate(generateRandomString());
    });
  };

  const handleSendExtend = () => {
    if (!selectedSendUser || !selectedReceiveRelay || !selectedExtendToRelay || !selectedCircuit) {
      toast.error('Please select a user and both relays for extend. Also select a circuit');
      return;
    }
    sendExtend(selectedSendUser, selectedReceiveRelay, selectedExtendToRelay, selectedCircuit).then(() => {
      setUpdate(generateRandomString());
    });
  };

  const handleSendEstablishRendezvous = () => {
    if (!selectedSendUser || !selectedReceiveRelay || !selectedCircuit) {
      toast.error('Please select a user and a relay');
      return;
    }
    sendEstablishRendezvous(selectedSendUser, selectedReceiveRelay, selectedCircuit).then(() => {
      setUpdate(generateRandomString());
    });
  }

  const handleSendEstablishIntroduction = () => {
    if (!selectedSendUser || !selectedReceiveRelay || !selectedCircuit) {
      toast.error('Please select a user and a relay');
      return;
    }
    sendEstablishIntroduction(selectedSendUser, selectedReceiveRelay, selectedCircuit).then(() => {
      setUpdate(generateRandomString);
    });
  }

  const handleSendBegin = () => {
    if (!selectedSendUser || !selectedReceiveRelay || !selectedCircuit || !selectedBeginRelay) {
      toast.error('Please select a user, relay, circuit and cookie');
      return;
    }
    sendBegin(selectedSendUser, selectedReceiveRelay, selectedCircuit, selectedBeginRelay).then(() => {
      setUpdate(generateRandomString());
    });
  }

  const handleSendIntroduce1 = () => {
    if (!selectedSendUser || !selectedReceiveRelay || !selectedCircuit || !selectedCookie || !selectedIntroduction || !selectedStream || !forUser) {
      toast.error('Please select a user, relay, circuit, rendezvous relay, and provide a rendezvous cookie');
      return;
    }

    sendIntroduce1(
      selectedSendUser,
      selectedReceiveRelay,
      selectedIntroduction,
      selectedStream,
      selectedCookie,
      forUser.rsa_public_key,
      selectedCircuit
    ).then(() => {
      setUpdate(generateRandomString());
    });
  };

  const handleSendRendezvous1 = () => {
    if (!selectedSendUser || !selectedReceiveRelay || !selectedCircuit || !selectedCookie) {
      toast.error('Please select a user, relay, circuit, and provide a rendezvous cookie');
      return;
    }

    sendRendezvous1(
      selectedSendUser,
      selectedReceiveRelay,
      selectedCircuit,
      selectedCookie,
    ).then(() => {
      setUpdate(generateRandomString());
    });
  };

  const handleSendData = () => {
    if (!selectedSendUser || !selectedReceiveRelay || !selectedCircuit || !selectedCookie || !data) {
      toast.error('Please select a user, relay, circuit, and provide a rendezvous cookie and data');
      return;
    }

    sendData(
      selectedSendUser,
      selectedReceiveRelay,
      selectedCircuit,
      selectedCookie,
      data
    ).then(() => {
      setUpdate(generateRandomString());
    });
  };

  const setRelay = useCallback((event: React.MouseEvent<HTMLElement>, relayId: string) => {
    setRelays(currentRelays => {
      const r = currentRelays.find(r => r.id === relayId);
      if (r) {
        setSelectedData(r);
      }
      return currentRelays;
    });
    event.stopPropagation();
    setPopupPosition({ x: event.clientX, y: event.clientY });
  }, []);

  const setUser = useCallback((event: React.MouseEvent<HTMLElement>, userId: string) => {
    setUsers(currentUsers => {
      const u = currentUsers.find(u => u.id === userId);
      if (u) {
        setSelectedData(u);
      }
      return currentUsers;
    });
    event.stopPropagation();
    setPopupPosition({ x: event.clientX, y: event.clientY });
  }, []);

  return (
    <AppContainer>
      <ToastContainer autoClose={1000} position='top-left' />
      <Dashboard>
        <ConnectionLinesWrapper>
          <ConnectionLines users={users} relays={relays} positions={positions} />
        </ConnectionLinesWrapper>
        <CardsWrapper>
          <AnimatePresence>
            {users.map(user => (
              <Draggable key={`${user.id}-${user.logs.length}`} bounds="parent" defaultPosition={positions[user.id]} onStop={(e, data) => handleDrag(user.id, data)}>
                <CardContainer>
                  <Card
                    type='user'
                    item={user}
                    onClick={(event: React.MouseEvent<HTMLElement>) => {
                      setUser(event, user.id);
                    }}
                  />
                </CardContainer>
              </Draggable>
            ))}
            {relays.map(relay => (
              <Draggable key={`${relay.id}-${relay.logs.length}`} bounds="parent" defaultPosition={positions[relay.id]} onStop={(e, data) => handleDrag(relay.id, data)}>
                <CardContainer>
                  <Card
                    isRendezvous={isRendezvousRelays[relay.id]}
                    isIntroduction={isIntroductionRelays[relay.id]}
                    item={relay}
                    type='relay'
                    onClick={(event: React.MouseEvent<HTMLElement>) => {
                      setRelay(event, relay.id);
                    }}
                  />
                </CardContainer>
              </Draggable>
            ))}
          </AnimatePresence>
        </CardsWrapper>
      </Dashboard>

      {selectedData && (
        <DataPopup
          data={selectedData}
          position={popupPosition}
        />
      )}

      <ControlPanel>
        <ControlPanelContent>
          <ButtonGroup>
            <TopButton onClick={() => { setUpdate(generateRandomString()); toast.success("Restarted Succesfully") }} title="Restart">
              <FiRefreshCw />
            </TopButton>
            <TopButton onClick={() => { window.open("https://www.vicyyn.com", "_blank") }} title="Info">
              <FiInfo />
            </TopButton>
            <TopButton onClick={() => { window.open("https://www.github.com/vicyyn/veilcomm", "_blank") }} title="Github">
              <FiGithub />
            </TopButton>
          </ButtonGroup>
          <Section>
            <SectionTitle>New Entities</SectionTitle>
            <Button onClick={() => handleStartUser()}>New User</Button>
            <Button onClick={() => handleStartRelay()}>New Relay</Button>
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
              value={selectedCircuit}
              onChange={(e) => setSelectedCircuit(e.target.value)}
            >
              <option value="">Select Circuit</option>
              {Object.keys(users.find(u => u.id === selectedSendUser?.id)?.circuits || {}).map(circuit => (
                <option key={circuit} value={circuit}>{circuit}</option>
              ))}
            </Select>
            <Select
              value={selectedCookie}
              onChange={(e) => setSelectedCookie(e.target.value)}
            >
              <option value="">Select Rendezvous Cookie</option>
              {Object.keys(users.find(u => u.id === selectedSendUser?.id)?.rendezvous_cookies || {}).map(cookie => (
                <option key={cookie} value={cookie}>{cookie}</option>
              ))}
            </Select>
            <Select
              value={selectedIntroduction}
              onChange={(e) => setSelectedIntroduction(e.target.value)}
            >
              <option value="">Select Introduction Id</option>
              {users.map(user =>
                Object.entries(user.introduction_points).map(([introduction_point, relayId]) => (
                  <option key={introduction_point} value={introduction_point}>{introduction_point}</option>
                ))
              )}
            </Select>
            <Select
              value={selectedStream}
              onChange={(e) => setSelectedStream(e.target.value)}
            >
              <option value="">Select Stream</option>
              {Object.keys(users.find(u => u.id === selectedSendUser?.id)?.streams || {}).map(circuit => (
                <option key={circuit} value={circuit}>{circuit}</option>
              ))}
            </Select>
          </Section>

          <Section>
            <SectionTitle>Send Create</SectionTitle>
            <Select
              value={selectedCreateRelay ? selectedCreateRelay.nickname : ''}
              onChange={(e) => setSelectedCreateRelay(relays.find(r => r.nickname === e.target.value))}
            >
              <option value="">Select Relay to Create Circuit</option>
              {relays.map(relay => (
                <option key={relay.nickname} value={relay.nickname}>{relay.nickname}</option>
              ))}
            </Select>
            <Button onClick={handleSendCreate}>Send Create</Button>
          </Section>


          <Section>
            <SectionTitle>Send Extend</SectionTitle>
            <Select
              value={selectedExtendToRelay ? selectedExtendToRelay.nickname : ''}
              onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setSelectedExtendToRelay(relays.find(r => r.nickname === e.target.value))}
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
              onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setForUser(users.find(u => u.nickname === e.target.value))}
            >
              <option value="">Select User to Communicate with</option>
              {users.map(user => (
                <option key={user.nickname} value={user.nickname}>{user.nickname}</option>
              ))}
            </Select>
            <Button onClick={handleSendIntroduce1}>Send Introduce 1</Button>
          </Section>

          <Section>
            <SectionTitle>Send Rendezvous1</SectionTitle>
            <Button onClick={handleSendRendezvous1}>Send Rendezvous1</Button>
          </Section>

          <Section>
            <SectionTitle>Send Data</SectionTitle>
            <Input
              type="text"
              placeholder="Enter message to send"
              value={data}
              onChange={(e) => setData(e.target.value)}
            />
            <Button onClick={handleSendData}>Send Data</Button>
          </Section>
        </ControlPanelContent>
      </ControlPanel>
    </AppContainer >
  );
}

export default App;