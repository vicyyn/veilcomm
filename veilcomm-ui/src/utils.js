const generateRandomPosition = () => {
  const x = Math.random() * 1000; // Random X position within 100px
  const y = Math.random() * 1000; // Random Y position within 100px
  return { x, y };
};

export { generateRandomPosition };
