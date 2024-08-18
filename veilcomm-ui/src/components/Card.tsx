import React from 'react';
import { motion } from "framer-motion";
import styled from "styled-components";
import { UserState, RelayState } from '../data';

interface StyledCardProps {
  $isUser: boolean;
}

const StyledCard = styled(motion.div) <StyledCardProps>`
  background-color: ${props => props.$isUser ? '#e6f7ff' : '#f6ffed'};
  border-radius: 8px;
  padding: 15px;
  cursor: pointer;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
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
      layout
      initial={{ opacity: 0, scale: 0.8 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.8 }}
      whileHover={{ scale: 1.05 }}
      whileTap={{ scale: 0.95 }}
      onClick={onClick}
      style={{ border: isSelected ? `2px solid ${type === 'user' ? '#1890ff' : '#52c41a'}` : 'none' }}
    >
      <h3>{item.nickname}</h3>
      <p>ID: {item.id}</p>
    </StyledCard>
  );
}

export default Card;