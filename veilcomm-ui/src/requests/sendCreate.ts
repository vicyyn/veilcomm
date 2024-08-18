import { toast } from "react-toastify";
import { RelayState, UserState } from "../data";

async function sendCreate(user: UserState, relay: RelayState): Promise<string | undefined> {
  if (!user || !relay) {
    toast.error('Please select a user and relay to send create');
    return undefined;
  }

  try {
    const response = await fetch(`http://127.0.0.1:8081/users/${user.id}/send_create`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        relay_id: relay.id
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
  return undefined
};

export default sendCreate;
