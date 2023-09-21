import { PublicUser } from 'bindings/PublicUser';
import ResizePanel from 'components/Atoms/ResizePanel';
import { useAllUsers } from 'data/AllUsers';
import { BrowserLocationContext } from 'data/BrowserLocationContext';
import { SettingsContext } from 'data/SettingsContext';
import { useUserInfo } from 'data/UserInfo';
import { useContext, useEffect, useState } from 'react';
import { Outlet } from 'react-router-dom';
import { useLocalStorage } from 'usehooks-ts';
import { useQueryParam } from 'utils/hooks';
import SettingsLeftNav from './SettingsLeftNav';

export const SettingsLayout = () => {
  const { data: userInfo } = useUserInfo();
  const canManageUsers = userInfo?.is_owner || false;
  const { data: dataUserList } = useAllUsers(canManageUsers);
  const [tabIndex, setTabIndex] = useState(0);
  const [leftNavSize, setLeftNavSize] = useLocalStorage('leftNavSize', 220);
  /* Start userList */
  const [queryUid, setQueryUid] = useQueryParam('user', '');
  const [selectedUser, setSelectedUser] = useState<PublicUser | null>(
    null
  );
  const userList = canManageUsers ? dataUserList : undefined;
  useEffect(() => {
    if (queryUid && userList && queryUid in userList) {
      setSelectedUser(userList[queryUid]);
    } else setSelectedUser(null);
  }, [userList, queryUid]);

  function selectUser(user: PublicUser | null) {
    console.log('selectUser', user);
    if (user === null) {
      setSelectedUser(null);
      setQueryUid('');
    } else {
      setSelectedUser(user);
      setQueryUid(user.uid);
    }
  }
  /* End userList */

  return (
    <SettingsContext.Provider
      value={{
        selectedUser,
        selectUser,
        userList: userList || {},
        tabIndex,
        setTabIndex,
      }}
    >
      <ResizePanel
        direction="e"
        maxSize={280}
        minSize={200}
        size={leftNavSize}
        validateSize={false}
        onResize={setLeftNavSize}
        containerClassNames="min-h-0"
      >
        <SettingsLeftNav className="border-r border-fade-700 bg-gray-850" />
      </ResizePanel>
      <div className="h-full min-w-0 grow child:h-full">
        <Outlet />
      </div>
    </SettingsContext.Provider>
  );
};
