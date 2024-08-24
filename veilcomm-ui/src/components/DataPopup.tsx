import React from 'react';
import styled from 'styled-components';
import { Position, RelayState, UserState } from '../data';

const PopupContainer = styled.div`
  position: fixed;
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  z-index: 100;
  width: 300px;
  max-height: 400px;
  display: flex;
  flex-direction: column;
  font-family: 'Arial', sans-serif;
  overflow: hidden;
  font-size: 12px;
`;

const Title = styled.h3`
  margin: 0;
  color: #333;
  border-bottom: 1px solid #f0f0f0;
  padding: 10px;
  background-color: #f8f8f8;
  font-size: 14px;
`;

const ScrollableContent = styled.div`
  flex-grow: 1;
  overflow-y: auto;
  padding: 10px;
`;

const DataSection = styled.div`
  margin-bottom: 10px;
`;

const DataKey = styled.strong`
  color: #555;
  font-size: 12px;
  display: block;
  margin-bottom: 2px;
`;

const DataValue = styled.pre`
  background-color: #f8f8f8;
  border-radius: 3px;
  padding: 4px;
  margin: 2px 0;
  font-size: 11px;
  white-space: pre-wrap;
  word-break: break-all;
  color: #333;
  border-left: 2px solid #3498db;
  overflow-y: auto;
`;

const LogsContainer = styled.div`
  margin-top: 10px;
  padding: 5px;
  background-color: #f0f0f0;
  border-radius: 4px;
  max-height: 150px;
  overflow-y: auto;
`;

const LogEntry = styled.pre`
  margin: 0;
  padding: 2px 0;
  font-size: 10px;
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
      return toHexString(value).substring(0, 15) + '...';
    }
    if (key === 'handshakes' || key === 'connected_users') {
      const hexObject = Object.fromEntries(
        Object.entries(value).map(([k, v]) => [k, toHexString(v as number[]).substring(0, 15) + '...'])
      );
      return JSON.stringify(hexObject, null, 1);
    }
    if (typeof value === 'object' && value !== null) {
      return JSON.stringify(value, null, 1);
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