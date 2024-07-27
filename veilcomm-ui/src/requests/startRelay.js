async function startRelay(relayNickname) {
  if (relayNickname) {
    alert('Please enter both nickname and address for the new relay');
    return;
  }
  try {
    const response = await fetch('http://127.0.0.1:8081/start_relay', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        nickname: relayNickname,
      }),
    });
    if (response.ok) {
      alert('Relay started successfully');
    } else {
      alert('Failed to start relay');
    }
  } catch (error) {
    console.error('Error:', error);
    alert('An error occurred while starting the relay');
  }
};

export default startRelay;