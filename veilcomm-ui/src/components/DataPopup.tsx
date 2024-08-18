import React from 'react';
import styled from 'styled-components';
import { Position, RelayState, UserState } from '../data';

const PopupContainer = styled.div`
  position: fixed;
  background-color: white;
  border-radius: 12px;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.1);
  z-index: 100;
  width: 400px;
  max-height: 600px;
  display: flex;
  flex-direction: column;
  font-family: 'Arial', sans-serif;
  overflow: hidden;
`;

const Title = styled.h3`
  margin: 0;
  color: #333;
  border-bottom: 2px solid #f0f0f0;
  padding: 20px;
  background-color: #f8f8f8;
`;

const ScrollableContent = styled.div`
  flex-grow: 1;
  overflow-y: auto;
  padding: 20px;
`;

const DataSection = styled.div`
  margin-bottom: 15px;
`;

const DataKey = styled.strong`
  color: #555;
  font-size: 14px;
  display: block;
  margin-bottom: 5px;
`;

const DataValue = styled.pre`
  background-color: #f8f8f8;
  border-radius: 4px;
  padding: 8px;
  margin: 5px 0;
  font-size: 13px;
  white-space: pre-wrap;
  word-break: break-all;
  color: #333;
  border-left: 3px solid #3498db;
  overflow-y: auto;
`;

const LogsContainer = styled.div`
  margin-top: 20px;
  padding: 10px;
  background-color: #f0f0f0;
  border-radius: 8px;
  max-height: 200px;
  overflow-y: auto;
`;

const LogEntry = styled.pre`
  margin: 0;
  padding: 4px 0;
  font-size: 12px;
  color: #555;
  border-bottom: 1px solid #e0e0e0;
  &:last-child {
    border-bottom: none;
  }
`;

const toHexString = (arr: number[]): string => {
  return Array.from(arr)
    .map(byte => byte.toString(16).padStart(2, '0'))
    .join('');
};

type DataPopupProps = {
  data: UserState | RelayState;
  position: Position;
};

function DataPopup({ data, position }: DataPopupProps): JSX.Element {
  const renderValue = (key: string, value: any): string => {
    if (key === 'rsa_public_key') {
      return toHexString(value).substring(0, 20) + '...';
    }
    if (key === 'handshakes' || key === 'connected_users') {
      const hexObject = Object.fromEntries(
        Object.entries(value).map(([k, v]) => [k, toHexString(v as number[]).substring(0, 20) + '...'])
      );
      return JSON.stringify(hexObject, null, 2);
    }
    if (typeof value === 'object' && value !== null) {
      return JSON.stringify(value, null, 2);
    }
    return String(value);
  };

  return (
    <PopupContainer style={{ top: position.y, left: position.x }}>
      <Title>{data.nickname}</Title>
      <ScrollableContent>
        {Object.entries(data).map(([key, value]) => {
          if (key === 'position' || key === 'nickname' || key === 'logs') return null;
          return (
            <DataSection key={key}>
              <DataKey>{key}:</DataKey>
              <DataValue>{renderValue(key, value)}</DataValue>
            </DataSection>
          );
        })}
        <LogsContainer>
          {data.logs.map((log, index) => (
            <LogEntry key={index}>{log}</LogEntry>
          ))}
        </LogsContainer>
      </ScrollableContent>
    </PopupContainer>
  );
}

export default DataPopup;