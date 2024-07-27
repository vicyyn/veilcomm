import { generateRandomPosition } from '../utils';

async function fetchRelays() {
  try {
    const response = await fetch('http://127.0.0.1:8081/relays');
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    const data = await response.json();
    const relaysWithPositions = data.map(relay => ({
      ...relay,
      position: generateRandomPosition()
    }));
    return relaysWithPositions
  } catch (error) {
    console.error('Error fetching relays:', error);
  }
};

export default fetchRelays;