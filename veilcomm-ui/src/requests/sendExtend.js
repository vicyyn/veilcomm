async function sendExtend(user, relay, extendRelay) {
  try {
    const response = await fetch(`http://127.0.0.1:8081/users/${user.id}/send_extend_to_relay/`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        relay_socket: relay.address,
        extend_to: extendRelay.address
      }),
    });
    if (response.ok) {
      alert('Extend sent successfully');
    } else {
      alert('Failed to send extend');
    }
  } catch (error) {
    console.error('Error:', error);
    alert('An error occurred while sending extend');
  }
};

export default sendExtend;
