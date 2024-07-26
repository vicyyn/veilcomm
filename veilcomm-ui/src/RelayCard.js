import React from 'react';
import { motion } from 'framer-motion';
import styled from 'styled-components';

const Card = styled(motion.div)`
  background-color: #f6ffed;
  border-radius: 8px;
  padding: 15px;
  cursor: pointer;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
`;

const RelayCard = ({ relay, isSelected, onClick }) => {
  return (
    <Card
      layout
      initial={{ opacity: 0, scale: 0.8 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.8 }}
      whileHover={{ scale: 1.05 }}
      whileTap={{ scale: 0.95 }}
      onClick={onClick}
      style={{ border: isSelected ? '2px solid #52c41a' : 'none' }}
    >
      <h3>{relay.nickname}</h3>
      <p>Address: {relay.address}</p>
    </Card>
  );
};

export default RelayCard;