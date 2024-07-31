import { toast } from "react-toastify";

async function sendBegin(user, relay, circuit_id, begin_relay) {
  if (!user || !relay || !circuit_id || !begin_relay) {
    toast.error('Please select a user, circuit_id, begin_relay and relay to send begin');
    return;
  }

  try {
    const response = await fetch(`http://127.0.0.1:8081/users/${user.id}/send_begin_to_relay`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        relay_socket: relay.address,
        circuit_id: circuit_id,
        begin_relay_socket: begin_relay.address
      }),
    });
    if (response.ok) {
      toast.success('Begin sent successfully');
      return response.json();
    } else {
      toast.error('Failed to send Begin');
    }
  } catch (error) {
    console.error('Error:', error);
    toast.error('An error occurred while sending Begin');
  }
};

export default sendBegin
