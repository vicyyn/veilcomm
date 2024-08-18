import React from 'react';
import { motion } from "framer-motion";
import styled from "styled-components";
import { UserState, RelayState } from '../data';

interface StyledCardProps {
  $isUser: boolean;
  $isSelected: boolean;
}

const StyledCard = styled(motion.div) <StyledCardProps>`
  height: 120px;
  width: 250px;
  background-color: ${props => props.$isUser ? '#e6f7ff' : '#f6ffed'};
  border-radius: 8px;
  padding: 15px;
  cursor: pointer;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  text-align: center;
  border: ${props => props.$isSelected ? `2px solid ${props.$isUser ? '#1890ff' : '#52c41a'}` : 'none'};
  transition: all 0.3s ease;
  z-index: 10; // Ensure this is higher than the SVGContainer's z-index
  &:hover {
    box-shadow: 0 6px 8px rgba(0, 0, 0, 0.15);
  }
`;

const Title = styled.h3`
  margin: 0 0 10px 0;
  font-size: 18px;
  color: #333;
`;

const IdText = styled.p`
  margin: 0;
  font-size: 14px;
  color: #666;
`;

type CardProps = {
  item: UserState | RelayState;
  onClick: (event: React.MouseEvent<HTMLElement>) => void;
  isSelected: boolean;
  type: 'user' | 'relay';
};

function Card({ item, onClick, isSelected, type }: CardProps): JSX.Element {
  const isUser = type === 'user';

  return (
    <StyledCard
      $isUser={isUser}
      $isSelected={isSelected}
      layout
      initial={{ opacity: 0, scale: 0.8 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.8 }}
      whileHover={{ scale: 1.05 }}
      whileTap={{ scale: 0.95 }}
      onClick={onClick}
    >
      <Title>{item.nickname}</Title>
      <IdText>ID: {item.id}</IdText>
    </StyledCard>
  );
}

export default Card;