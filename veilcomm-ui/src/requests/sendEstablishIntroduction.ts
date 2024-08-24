import { toast } from "react-toastify";
import { RelayState, UserState } from "../data";

async function sendEstablishIntroduction(user: UserState, relay: RelayState, circuit_id: string) {
  if (!user || !relay || !circuit_id) {
    toast.error('Please select a user, circuit_id and relay to send establish introduction');
    return;
  }

  try {
    const response = await fetch(`http://127.0.0.1:8081/users/${user.id}/send_establish_introduction`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        relay_id: relay.id,
        circuit_id: circuit_id
      }),
    });
    if (response.ok) {
      toast.success('Establish Introduction sent successfully');
    } else {
      toast.error('Failed to send Establish Introduction');
    }
  } catch (error) {
    console.error('Error:', error);
    toast.error('An error occurred while sending Establish Introduction');
  }
};

export default sendEstablishIntroduction;
