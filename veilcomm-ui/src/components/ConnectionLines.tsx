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
  cardSize: { width: number; height: number };
}

const ConnectionLines: React.FC<Props> = ({ users, relays, positions, cardSize }) => {
  const [lines, setLines] = useState<JSX.Element[]>([]);

  const getCircuitColor = useCallback(() => {
    const colors = [
      '#E6772E', // Dark Orange
      '#C23B22', // Vermilion
      '#D6618F', // Dark Pink
      '#8E4585', // Plum
      '#4A5899', // Dark Blue-Purple
      '#38764A', // Forest Green
      '#31A354', // Emerald
      '#9D5C0D', // Bronze
      '#D49F39', // Golden Brown
      '#B75D69', // Dark Rose
      '#766EC8', // Blue-Violet
      '#6E8E84', // Dark Sea Green
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

    users.forEach((user) => {
      Object.entries(user.circuits).forEach(([circuitId, relayIds]) => {
        const color = circuitColorGetter(circuitId);

        // Connect the user to the first relay
        const userPos = getCardCenter(positions[user.id]);
        const firstRelayPos = getCardCenter(positions[relayIds[0]]);

        if (userPos && firstRelayPos) {
          newLines.push(
            <g key={`${user.id}-${circuitId}-start`}>
              <line
                x1={userPos.x}
                y1={userPos.y}
                x2={firstRelayPos.x}
                y2={firstRelayPos.y}
                stroke={color}
                strokeWidth="2"
              />
            </g>
          );
        }

        // Connect the relays
        for (let i = 0; i < relayIds.length - 1; i++) {
          const startPos = getCardCenter(positions[relayIds[i]]);
          const endPos = getCardCenter(positions[relayIds[i + 1]]);

          if (startPos && endPos) {
            newLines.push(
              <g key={`${user.id}-${circuitId}-${i}`}>
                <line
                  x1={startPos.x}
                  y1={startPos.y}
                  x2={endPos.x}
                  y2={endPos.y}
                  stroke={color}
                  strokeWidth="2"
                />
              </g>
            );
          }
        }
      });
    });

    setLines(newLines);
  }, [users, relays, positions, cardSize, getCircuitColor]);

  return <SVGContainer>{lines}</SVGContainer>;
};

export default ConnectionLines;