import { toast } from "react-toastify";
async function fetchUserRelays(users) {
  for (const user of users) {
    try {
      const response = await fetch(`http://127.0.0.1:8081/users/${user.id}/fetch_relays`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
    } catch (error) {
      console.error('Error fetching user relays:', error);
      toast.error('Failed to fetch user relays');
    }
  }
};

export default fetchUserRelays;