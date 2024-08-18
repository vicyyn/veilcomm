import { toast } from "react-toastify";
async function startRelay(relayNickname: string) {
  if (!relayNickname) {
    toast.error('Please enter nickname for the new relay');
    return;
  }
  try {
    const response = await fetch('http://127.0.0.1:8081/start_relay', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        nickname: relayNickname,
      }),
    });
    if (response.ok) {
      toast.success('Relay started successfully');
    } else {
      toast.error('Failed to start relay');
    }
  } catch (error) {
    console.error('Error:', error);
    toast.error('An error occurred while starting the relay');
  }
};

export default startRelay;