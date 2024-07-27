async function startUser(userNickname) {
  if (!userNickname) {
    alert('Please enter a nickname for the new user');
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
      alert('User started successfully');
    } else {
      alert('Failed to start user');
    }
  } catch (error) {
    console.error('Error:', error);
    alert('An error occurred while starting the user');
  }
};

export default startUser;