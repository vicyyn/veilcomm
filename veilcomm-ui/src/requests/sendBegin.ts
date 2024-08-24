import { toast } from "react-toastify";
import { RelayState, UserState } from "../data";

async function sendBegin(user: UserState, relay: RelayState, circuit_id: string, begin_relay: RelayState) {
  if (!user || !relay || !circuit_id || !begin_relay) {
    toast.error('Please select a user, circuit_id, begin_relay and relay to send begin');
    return;
  }

  try {
    const response = await fetch(`http://127.0.0.1:8081/users/${user.id}/send_begin`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        relay_id: relay.id,
        circuit_id: circuit_id,
        begin_relay_id: begin_relay.id
      }),
    });
    if (response.ok) {
      toast.success('Begin sent successfully');
    } else {
      toast.error('Failed to send Begin');
    }
  } catch (error) {
    console.error('Error:', error);
    toast.error('An error occurred while sending Begin');
  }
};

export default sendBegin
