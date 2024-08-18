import { ApiState } from '../data';

async function getState(): Promise<ApiState | undefined> {
  try {
    const response = await fetch('http://127.0.0.1:8081/get_state', {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (response.ok) {
      const data: ApiState = await response.json();
      return data;
    } else {
      return undefined;
    }
  } catch (error) {
    console.error('Error:', error);
    return undefined;
  }
}

export default getState;