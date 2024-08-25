import React from 'react';
import { motion } from "framer-motion";
import styled from "styled-components";
import { UserState, RelayState } from '../data';

interface StyledCardProps {
  $isUser: boolean;
  $isRendezvous: boolean;
  $isIntroduction: boolean;
}

const StyledCard = styled(motion.div) <StyledCardProps>`
  height: ${props => props.$isRendezvous || props.$isIntroduction ? '70px' : '60px'};
  width: ${props => props.$isRendezvous || props.$isIntroduction ? '160px' : '150px'};
  background-color: ${props =>
    props.$isUser ? '#e6f7ff' :
      props.$isRendezvous ? '#fffbe6' :
        props.$isIntroduction ? '#fff0f6' :
          '#f6ffed'};
  border-radius: ${props => props.$isRendezvous || props.$isIntroduction ? '12px' : '8px'};
  padding: 15px;
  cursor: pointer;
  box-shadow: ${props =>
    props.$isRendezvous ? '0 6px 12px rgba(250, 173, 20, 0.3)' :
      props.$isIntroduction ? '0 6px 12px rgba(235, 47, 150, 0.3)' :
        '0 4px 6px rgba(0, 0, 0, 0.1)'};
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  text-align: center;
  border: ${props =>
    props.$isRendezvous ? '2px solid #faad14' :
      props.$isIntroduction ? '2px solid #eb2f96' :
        props.$isUser ? '#1890ff'
          : 'none'};
  transition: all 0.3s ease;
  z-index: ${props => props.$isRendezvous || props.$isIntroduction ? 20 : 10};
  position: relative;
  transform: ${props => props.$isRendezvous || props.$isIntroduction ? 'rotate(-2deg)' : 'none'};

  &:hover {
    box-shadow: ${props =>
    props.$isRendezvous ? '0 8px 16px rgba(250, 173, 20, 0.4)' :
      props.$isIntroduction ? '0 8px 16px rgba(235, 47, 150, 0.4)' :
        '0 6px 8px rgba(0, 0, 0, 0.15)'};
    transform: ${props => props.$isRendezvous || props.$isIntroduction ? 'rotate(-2deg) scale(1.05)' : 'scale(1.05)'};
  }
`;

const Title = styled.h3<{ $isSpecial: boolean }>`
  font-size: 12px;
  color: ${props => props.$isSpecial ? '#333' : '#666'};
  margin: 0;
  font-weight: bold;
`;

const IdText = styled.p<{ $isSpecial: boolean }>`
  margin: 5px 0 0;
  font-size: 10px;
`;

type CardProps = {
  item: UserState | RelayState;
  onClick: (event: React.MouseEvent<HTMLElement>) => void;
  isRendezvous?: boolean;
  isIntroduction?: boolean;
  type: 'user' | 'relay';
};

function Card({ item, onClick, type, isRendezvous = false, isIntroduction = false }: CardProps): JSX.Element {
  const isUser = type === 'user';
  const isSpecial = isRendezvous || isIntroduction;

  return (
    <StyledCard
      $isUser={isUser}
      $isRendezvous={isRendezvous}
      $isIntroduction={isIntroduction}
      layout
      initial={{ opacity: 0, scale: 0.8 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.8 }}
      whileHover={{ scale: isSpecial ? 1.1 : 1.05 }}
      whileTap={{ scale: 0.95 }}
      onClick={onClick}
    >
      <Title $isSpecial={isSpecial}>
        {isRendezvous ? 'Rendezvous: ' : isIntroduction ? 'Introduction: ' : ''}{item.nickname}
      </Title>
      <IdText $isSpecial={isSpecial}>ID: {item.id}</IdText>
    </StyledCard>
  );
}

export default Card;