import { generateRandomPosition } from "../utils";

async function fetchUsers() {
  try {
    const response = await fetch('http://127.0.0.1:8081/users');
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    const data = await response.json();
    console.log(data)
    const usersWithPositions = data.map(user => ({
      ...user,
      position: generateRandomPosition()
    }));
    return usersWithPositions
  } catch (error) {
    console.error('Error fetching users:', error);
    return null
  }
}

export default fetchUsers;