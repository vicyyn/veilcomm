function generateRandomPosition() {
  const x = Math.random() * 1000; // Random X position within 100px
  const y = Math.random() * 1000; // Random Y position within 100px
  return { x, y };
};

function generateRandomString() {
  const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let result = '';
  for (let i = 0; i < 10; i++) {
    result += characters.charAt(Math.floor(Math.random() * characters.length));
  }
  return result;
}

export { generateRandomPosition, generateRandomString };
