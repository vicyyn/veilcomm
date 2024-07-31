import { toast } from "react-toastify";

async function sendEstablishIntroduction(user, relay, circuit_id) {
  if (!user || !relay || !circuit_id) {
    toast.error('Please select a user, circuit_id and relay to send establish introduction');
    return;
  }

  try {
    const response = await fetch(`http://127.0.0.1:8081/users/${user.id}/send_establish_introduction_to_relay`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        relay_socket: relay.address,
        circuit_id: circuit_id
      }),
    });
    if (response.ok) {
      toast.success('Establish Introduction sent successfully');
      return response.json();
    } else {
      toast.error('Failed to send Establish Introduction');
    }
  } catch (error) {
    console.error('Error:', error);
    toast.error('An error occurred while sending Establish Introduction');
  }
};

export default sendEstablishIntroduction;
