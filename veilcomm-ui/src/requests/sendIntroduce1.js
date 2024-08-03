import { toast } from "react-toastify";

async function sendIntroduce1(user, relay, introductionId, streamId, rendezvousPointRelay, rendezvousCookie, introductionRsaPublic, circuitId) {
  if (!user || !relay || !introductionId || !streamId || !rendezvousPointRelay || !rendezvousCookie || !introductionRsaPublic || !circuitId) {
    toast.error('All parameters are required to send Introduce1');
    return;
  }

  try {
    const response = await fetch(`http://127.0.0.1:8081/users/${user.id}/send_introduce1_to_relay`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        relay_socket: relay.address,
        introduction_id: introductionId,
        stream_id: streamId,
        rendezvous_point_socket: rendezvousPointRelay.address,
        rendezvous_cookie: rendezvousCookie,
        introduction_rsa_public: Array.from(introductionRsaPublic),
        circuit_id: circuitId
      }),
    });

    if (response.ok) {
      toast.success('Introduce1 sent successfully');
    } else {
      const errorText = await response.text();
      toast.error(`Failed to send Introduce1: ${errorText}`);
      console.error('Server response:', response.status, errorText);
    }
  } catch (error) {
    console.error('Error:', error);
    toast.error('An error occurred while sending Introduce1');
  }
}

export default sendIntroduce1;