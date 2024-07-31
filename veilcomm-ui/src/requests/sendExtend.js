import { toast } from "react-toastify";

async function sendExtend(user, relay, extendRelay, circuit_id) {
  if (!user || !relay || !extendRelay || !circuit_id) {
    toast.error('Please select a user, relay, extend relay, and circuit to send extend');
    return;
  }
  try {
    const response = await fetch(`http://127.0.0.1:8081/users/${user.id}/send_extend_to_relay`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        relay_socket: relay.address,
        extend_to: extendRelay.address,
        circuit_id: circuit_id
      }),
    });
    if (response.ok) {
      toast.success('Extend sent successfully');
    } else {
      toast.error('Failed to send extend');
    }
  } catch (error) {
    console.error('Error:', error);
    toast.error('An error occurred while sending extend');
  }
};

export default sendExtend;
