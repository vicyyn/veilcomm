import RelayState from "./relay";
import UserState from "./user";

export default interface ApiState {
  relay_states: RelayState[];
  user_states: UserState[];
}