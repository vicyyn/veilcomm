import { toast } from "react-toastify";

async function sendRendezvous1(user, relay, circuitId, rendezvousCookie) {
  if (!user || !relay || !rendezvousCookie || !circuitId) {
    toast.error('All parameters are required to send Sen');
    return;
  }

  try {
    const response = await fetch(`http://127.0.0.1:8081/users/${user.id}/send_rendezvous1_to_relay`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        relay_socket: relay.address,
        rendezvous_cookie: rendezvousCookie,
        circuit_id: circuitId
      }),
    });

    if (response.ok) {
      toast.success('Rendezvous1 sent successfully');
    } else {
      const errorText = await response.text();
      toast.error(`Failed to send Rendezvous1: ${errorText}`);
      console.error('Server response:', response.status, errorText);
    }
  } catch (error) {
    console.error('Error:', error);
    toast.error('An error occurred while sending Rendezvous1');
  }
}

export default sendRendezvous1;