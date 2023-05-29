import {
  Box,
  Avatar,
  Typography,
  Card,
  styled,
  Divider,
  useTheme,
} from "@mui/material";
import {
  formatDistance,
  format,
  subDays,
  subHours,
  subMinutes,
} from "date-fns";
import ScheduleTwoToneIcon from "@mui/icons-material/ScheduleTwoTone";

const CardWrapperPrimary = styled(Card)(
  ({ theme }) => `
      background: ${theme.colors.primary.main};
      color: ${theme.palette.primary.contrastText};
      padding: ${theme.spacing(2)};
      border-radius: ${theme.general.borderRadiusXl};
      border-top-right-radius: ${theme.general.borderRadius};
      max-width: 380px;
      display: inline-flex;
`
);

const CardWrapperSecondary = styled(Card)(
  ({ theme }) => `
      background: ${theme.palette.grey[200]};
      color: ${theme.colors.alpha.black[100]};
      padding: ${theme.spacing(2)};
      border-radius: ${theme.general.borderRadiusXl};
      border-top-left-radius: ${theme.general.borderRadius};
      max-width: 380px;
      display: inline-flex;
`
);

function ChatContent() {
  const theme = useTheme();

  return (
    <Box p={3} sx={{ border: `${theme.colors.alpha.black[50]} solid 2px` }}>
      <Box
        display="flex"
        alignItems="flex-start"
        justifyContent="flex-start"
        py={3}
      >
        <Box
          display="flex"
          alignItems="flex-start"
          flexDirection="column"
          justifyContent="flex-start"
          ml={2}
        >
          <CardWrapperSecondary>
            Hi. Can you send me the missing invoices asap?
          </CardWrapperSecondary>
        </Box>
      </Box>

      <Box
        display="flex"
        alignItems="flex-start"
        justifyContent="flex-end"
        py={3}
      >
        <Box
          display="flex"
          alignItems="flex-end"
          flexDirection="column"
          justifyContent="flex-end"
          mr={2}
        >
          <CardWrapperPrimary>
            Yes, I'll email them right now. I'll let you know once the remaining
            invoices are done.
          </CardWrapperPrimary>
        </Box>
      </Box>

      <Box
        display="flex"
        alignItems="flex-start"
        justifyContent="flex-end"
        py={3}
      >
        <Box
          display="flex"
          alignItems="flex-end"
          flexDirection="column"
          justifyContent="flex-end"
          mr={2}
        >
          <CardWrapperPrimary>Hey! Are you there?</CardWrapperPrimary>
          <CardWrapperPrimary
            sx={{
              mt: 2,
            }}
          >
            Heeeelloooo????
          </CardWrapperPrimary>
        </Box>
      </Box>
      <Box
        display="flex"
        alignItems="flex-start"
        justifyContent="flex-start"
        py={3}
      >
        <Box
          display="flex"
          alignItems="flex-start"
          flexDirection="column"
          justifyContent="flex-start"
          ml={2}
        >
          <CardWrapperSecondary>Hey there!</CardWrapperSecondary>
          <CardWrapperSecondary
            sx={{
              mt: 1,
            }}
          >
            How are you? Is it ok if I call you?
          </CardWrapperSecondary>
        </Box>
      </Box>
      <Box
        display="flex"
        alignItems="flex-start"
        justifyContent="flex-end"
        py={3}
      >
        <Box
          display="flex"
          alignItems="flex-end"
          flexDirection="column"
          justifyContent="flex-end"
          mr={2}
        >
          <CardWrapperPrimary>
            Hello, I just got my Amazon order shipped and I’m very happy about
            that.
          </CardWrapperPrimary>
          <CardWrapperPrimary
            sx={{
              mt: 1,
            }}
          >
            Can you confirm?
          </CardWrapperPrimary>
        </Box>
      </Box>
    </Box>
  );
}

export default ChatContent;
