import { toast } from "react-toastify";

async function sendData(user, relay, circuitId, rendezvousCookie, data) {
  if (!user || !relay || !rendezvousCookie || !circuitId || !data) {
    toast.error('All parameters are required to send Data');
    return;
  }
  // convert string to array
  data = Array.from(data).map(c => c.charCodeAt(0));
  console.log(data)

  try {
    const response = await fetch(`http://127.0.0.1:8081/users/${user.id}/send_data_to_relay`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        relay_socket: relay.address,
        rendezvous_cookie: rendezvousCookie,
        circuit_id: circuitId,
        data: data,
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