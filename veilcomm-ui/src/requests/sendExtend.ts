import { toast } from "react-toastify";
import { RelayState, UserState } from "../data";

async function sendExtend(user: UserState, relay: RelayState, extendRelay: RelayState, circuit_id: string) {
  if (!user || !relay || !extendRelay || !circuit_id) {
    toast.error('Please select a user, relay, extend relay, and circuit to send extend');
    return;
  }
  try {
    const response = await fetch(`http://127.0.0.1:8081/users/${user.id}/send_extend`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        relay_id: relay.id,
        extend_to_id: extendRelay.id,
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
