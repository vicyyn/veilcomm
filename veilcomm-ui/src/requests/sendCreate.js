import { toast } from "react-toastify";

async function sendCreate(user, relay) {
  if (!user || !relay) {
    toast.error('Please select a user and relay to send create');
    return;
  }

  console.log(user, relay)
  try {
    const response = await fetch(`http://127.0.0.1:8081/users/${user.id}/send_create_to_relay`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        relay_socket: relay.address
      }),
    });
    if (response.ok) {
      toast.success('Create sent successfully');
      return response.json();
    } else {
      toast.error('Failed to send create');
    }
  } catch (error) {
    console.error('Error:', error);
    toast.error('An error occurred while sending create');
  }
};

export default sendCreate;
