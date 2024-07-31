
async function getUserLogs() {
  try {
    const response = await fetch('http://127.0.0.1:8081/user_logs', {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      throw new Error('Failed to fetch user logs');
    }

    const logs = await response.json();
    return logs;
  } catch (error) {
    console.error('Error fetching user logs:', error);
    throw error;
  }
}

export default getUserLogs;