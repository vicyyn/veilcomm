import React from 'react';
import styled from 'styled-components';

const PopupContainer = styled.div`
  position: absolute;
  background-color: white;
  border-radius: 8px;
  padding: 15px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  z-index: 100;
  max-width: 400px;
  overflow-wrap: break-word;
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

const LogsContainer = styled.div`
  margin-top: 20px;
  padding: 10px;
  background-color: #f0f0f0;
  border-radius: 4px;
  max-height: 200px;
  overflow-y: auto;
`;

const LogEntry = styled.pre`
  margin: 0;
  padding: 2px 0;
  font-size: 14px;
  text-align: left;
`;


const DataPopup = ({ data, position, getLogs }) => {
  const [logs, setLogs] = React.useState([]);

  React.useEffect(() => {
    async function fetchData() {
      let logs = await getLogs();
      setLogs(logs);
    }
    fetchData();
  });

  const toHexString = (str) => {
    return Array.from(new TextEncoder().encode(str))
      .map(byte => byte.toString(16).padStart(2, '0'))
      .join('');
  };

  const renderValue = (key, value) => {
    if (key === 'rsa_public') {
      const hexString = toHexString(value);
      return hexString.substring(0, 20) + '...';
    }
    if (typeof value === 'object' && value !== null) {
      return JSON.stringify(value, null, 2);
    }
    return String(value);
  };

  return (
    <PopupContainer style={{ top: position.y, left: position.x }}>
      <h3>{data.nickname}</h3>
      {Object.entries(data).map(([key, value]) => {
        if (key === 'position' || key === 'nickname') return null;
        return (
          <div key={key}>
            <strong>{key}:</strong>
            <pre style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
              {renderValue(key, value)}
            </pre>
          </div>
        );
      })}
      <LogsContainer>
        {logs && logs.map((log, index) => (
          <LogEntry key={index}>{log}</LogEntry>
        ))}
      </LogsContainer>
    </PopupContainer>
  );
};

export default DataPopup;