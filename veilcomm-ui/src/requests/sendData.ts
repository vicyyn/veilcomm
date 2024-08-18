import { toast } from "react-toastify";
import { RelayState, UserState } from "../data";

async function sendData(user: UserState, relay: RelayState, circuitId: string, rendezvousCookie: string, data: string) {
  if (!user || !relay || !rendezvousCookie || !circuitId || !data) {
    toast.error('All parameters are required to send Data');
    return;
  }
  let newData = Array.from(data).map(c => c.charCodeAt(0));
  try {
    const response = await fetch(`http://127.0.0.1:8081/users/${user.id}/send_data`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        relay_id: relay.id,
        rendezvous_cookie: rendezvousCookie,
        circuit_id: circuitId,
        data: newData,
      }),
    });

    if (response.ok) {
      toast.success('Data sent successfully');
    } else {
      const errorText = await response.text();
      toast.error(`Failed to send Data: ${errorText}`);
      console.error('Server response:', response.status, errorText);
    }
  } catch (error) {
    console.error('Error:', error);
    toast.error('An error occurred while sending Data');
  }
}

export default sendData;