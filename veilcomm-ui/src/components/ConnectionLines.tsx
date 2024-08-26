import React, { useEffect, useState, useCallback } from 'react';
import styled from 'styled-components';
import { UserState, RelayState } from '../data';

const SVGContainer = styled.svg`
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 1;
`;

interface Position {
  x: number;
  y: number;
}

interface Props {
  users: UserState[];
  relays: RelayState[];
  positions: { [key: string]: Position };
  cardSize?: { width: number; height: number };
}

const ConnectionLines: React.FC<Props> = ({ users, relays, positions, cardSize = { 'height': 60, 'width': 150 } }) => {
  const [lines, setLines] = useState<JSX.Element[]>([]);

  const getCircuitColor = useCallback(() => {
    const colors = [
      '#E6772E', '#C23B22', '#D6618F', '#8E4585', '#4A5899', '#38764A',
      '#31A354', '#9D5C0D', '#D49F39', '#B75D69', '#766EC8', '#6E8E84',
    ];
    const cache: { [key: string]: string } = {};
    let colorIndex = 0;
    return (circuitId: string): string => {
      if (!cache[circuitId]) {
        cache[circuitId] = colors[colorIndex % colors.length];
        colorIndex++;
      }
      return cache[circuitId];
    };
  }, []);

  useEffect(() => {
    const newLines: JSX.Element[] = [];
    const circuitColorGetter = getCircuitColor();

    const getCardCenter = (pos: Position): Position => ({
      x: pos.x + cardSize.width / 2,
      y: pos.y + cardSize.height / 2,
    });

    // Draw lines for user-relay and relay-relay connections (unchanged)
    users.forEach((user) => {
      Object.entries(user.circuits).forEach(([circuitId, relayIds]) => {
        const color = circuitColorGetter(circuitId);

        // Connect the user to the first relay
        const userPos = getCardCenter(positions[user.id]);
        const firstRelayPos = getCardCenter(positions[relayIds[0]]);
        if (userPos && firstRelayPos) {
          newLines.push(
            <line
              key={`${user.id}-${circuitId}-start`}
              x1={userPos.x}
              y1={userPos.y}
              x2={firstRelayPos.x}
              y2={firstRelayPos.y}
              stroke={color}
              strokeWidth="2"
            />
          );
        }

        // Connect the relays
        for (let i = 0; i < relayIds.length - 1; i++) {
          const startPos = getCardCenter(positions[relayIds[i]]);
          const endPos = getCardCenter(positions[relayIds[i + 1]]);
          if (startPos && endPos) {
            newLines.push(
              <line
                key={`${user.id}-${circuitId}-${i}`}
                x1={startPos.x}
                y1={startPos.y}
                x2={endPos.x}
                y2={endPos.y}
                stroke={color}
                strokeWidth="2"
              />
            );
          }
        }
      });
    });

    // Draw dashed lines for stream connections
    relays.forEach((relay) => {
      Object.entries(relay.streams).forEach(([streamId, connectedRelayId]) => {
        const startPos = getCardCenter(positions[relay.id]);
        const endPos = getCardCenter(positions[connectedRelayId]);
        if (startPos && endPos) {
          const color = circuitColorGetter(streamId);
          newLines.push(
            <line
              key={`stream-${relay.id}-${streamId}-${connectedRelayId}`}
              x1={startPos.x}
              y1={startPos.y}
              x2={endPos.x}
              y2={endPos.y}
              stroke={color}
              strokeWidth="2"
              strokeDasharray="5,5"
            />
          );
        }
      });
    });

    setLines(newLines);
  }, [users, relays, positions, cardSize, getCircuitColor]);

  return <SVGContainer>{lines}</SVGContainer>;
};

export default ConnectionLines;