import { toast } from "react-toastify";

async function startUser(userNickname: string) {
  if (!userNickname) {
    toast.error('Please enter a nickname for the new user');
    return;
  }
  try {
    const response = await fetch('http://127.0.0.1:8081/start_user', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ nickname: userNickname }),
    });
    if (response.ok) {
      toast.success('User started successfully');
    } else {
      toast.error('Failed to start user');
    }
  } catch (error) {
    console.error('Error:', error);
    toast.error('An error occurred while starting the user');
  }
};

export default startUser;