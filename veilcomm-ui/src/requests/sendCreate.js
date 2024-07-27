async function sendCreate(user, relay) {
  try {
    const response = await fetch(`http://127.0.0.1:8081/users/${user.id}/send_create_to_relay/`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        relay_socket: relay.address
      }),
    });
    if (response.ok) {
      alert('Create sent successfully');
    } else {
      alert('Failed to send create');
    }
  } catch (error) {
    console.error('Error:', error);
    alert('An error occurred while sending create');
  }
};

export default sendCreate;
